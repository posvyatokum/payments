use crate::client::{Client, ClientView};
use crate::db::{Database, InMemoryDB};
use crate::transactions::{DepositTransaction, Transaction};
use crate::types::{ClientID, EngineError};
use log::warn;
use std::sync::Arc;

pub struct Engine {
    database: Arc<dyn Database>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            database: Arc::new(InMemoryDB::new()),
        }
    }

    pub fn process_transaction(&self, tx: &Transaction) -> Result<(), EngineError> {
        let client_id = tx.client();
        let mut client = self.get_client(&client_id)?;

        // Ignore transaction if the client is frozen.
        if client.is_frozen() {
            warn!(target: "engine", "Unable to process transaction {tx:?}. Client's account is frozen {client:?}.");
            return Ok(());
        }
        match tx {
            Transaction::Deposit(deposit) => {
                client.process_deposit(deposit)?;
            }
            Transaction::Withdrawal(withdrawal) => {
                client.process_withdrawal(withdrawal)?;
            }
            Transaction::Dispute(_) => {
                if let Some(referenced_deposit) = self.get_referenced_tx(tx)? {
                    client.process_dispute(&referenced_deposit)?;
                }
            }
            Transaction::Resolve(resolve) => {
                if !client.disputes.contains(&resolve.tx) {
                    warn!(target: "engine", "Cannot resolve transaction that is not disputed {resolve:?}.");
                } else if let Some(referenced_deposit) = self.get_referenced_tx(tx)? {
                    client.process_resolve(&referenced_deposit)?;
                }
            }
            Transaction::Chargeback(chargeback) => {
                if !client.disputes.contains(&chargeback.tx) {
                    warn!(target: "engine", "Cannot charge back transaction that is not disputed {chargeback:?}.");
                } else if let Some(referenced_deposit) = self.get_referenced_tx(tx)? {
                    client.process_chargeback(&referenced_deposit)?;
                }
            }
        }
        // Only record Deposit or Withdrawal transactions.
        if tx.is_recorded() {
            self.database.write_tx(tx.clone())?;
        }
        // Update Client entry in the db.
        self.database.write_client(client_id, client)?;
        Ok(())
    }

    /// Get client from db by ID, or create an empty client
    pub fn get_client(&self, id: &ClientID) -> Result<Client, EngineError> {
        Ok(match self.database.get_client(id)? {
            Some(client) => client.clone(),
            None => Client::default(),
        })
    }

    /// Get a vector of clients in output-friendly form
    pub fn get_all_clients(&self) -> Result<Vec<ClientView>, EngineError> {
        Ok(self.database.all_clients()?)
    }

    /// Get original transaction by id from meta-transaction.
    /// Does not panic if transaction is not in the db (returns None instead).
    /// If the transaction is not a Deposit, also returns None.
    fn get_referenced_tx(
        &self,
        tx: &Transaction,
    ) -> Result<Option<DepositTransaction>, EngineError> {
        let tx = self.database.get_tx(&tx.uid())?;
        match tx {
            None => {
                warn!("Disputed transaction is absent from DB. {:?}", tx);
                Ok(None)
            }
            Some(Transaction::Deposit(deposit)) => Ok(Some(deposit)),
            Some(ref other_tx) => {
                warn!(
                    "Disputed transaction is not a deposit. {:?} {:?}",
                    tx, other_tx
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::client::{Client, ClientStatus};
    use crate::engine::Engine;
    use crate::transactions::{
        ChargebackTransaction, DepositTransaction, DisputeTransaction, ResolveTransaction,
        Transaction, WithdrawalTransaction,
    };
    use rust_decimal_macros::dec;
    use std::collections::HashSet;

    #[test]
    fn test_deposit() {
        let engine = Engine::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        engine.process_transaction(&tx1).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(100.0),
                held: dec!(0.0),
                status: ClientStatus::Live,
                disputes: HashSet::new(),
            }
        );
    }

    #[test]
    fn test_withdrawal() {
        let engine = Engine::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        engine.process_transaction(&tx1).unwrap();
        let tx2 = Transaction::Withdrawal(WithdrawalTransaction {
            client: 10,
            tx: 2,
            amount: dec!(90.0),
        });
        engine.process_transaction(&tx2).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(10.0),
                held: dec!(0.0),
                status: ClientStatus::Live,
                disputes: HashSet::new(),
            }
        );
        let tx3 = Transaction::Withdrawal(WithdrawalTransaction {
            client: 10,
            tx: 3,
            amount: dec!(20.0),
        });
        engine.process_transaction(&tx3).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(10.0),
                held: dec!(0.0),
                status: ClientStatus::Live,
                disputes: HashSet::new(),
            }
        );
    }

    #[test]
    fn test_dispute() {
        let engine = Engine::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        engine.process_transaction(&tx1).unwrap();
        let tx2 = Transaction::Dispute(DisputeTransaction { client: 10, tx: 1 });
        engine.process_transaction(&tx2).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(0.0),
                held: dec!(100.0),
                status: ClientStatus::Live,
                disputes: HashSet::from([1]),
            }
        );
    }

    #[test]
    fn test_chargeback() {
        let engine = Engine::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        engine.process_transaction(&tx1).unwrap();
        let tx2 = Transaction::Dispute(DisputeTransaction { client: 10, tx: 1 });
        engine.process_transaction(&tx2).unwrap();
        let tx3 = Transaction::Chargeback(ChargebackTransaction { client: 10, tx: 1 });
        engine.process_transaction(&tx3).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(0.0),
                held: dec!(0.0),
                status: ClientStatus::Frozen,
                disputes: HashSet::new(),
            }
        );
    }

    #[test]
    fn test_resolve() {
        let engine = Engine::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        engine.process_transaction(&tx1).unwrap();
        let tx2 = Transaction::Dispute(DisputeTransaction { client: 10, tx: 1 });
        engine.process_transaction(&tx2).unwrap();
        let tx3 = Transaction::Resolve(ResolveTransaction { client: 10, tx: 1 });
        engine.process_transaction(&tx3).unwrap();
        assert_eq!(
            engine.get_client(&10).unwrap(),
            Client {
                available: dec!(100.0),
                held: dec!(0.0),
                status: ClientStatus::Live,
                disputes: HashSet::new(),
            }
        );
    }
}
