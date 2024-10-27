use crate::client::{Client, ClientView};
use crate::transactions::Transaction;
use crate::types::{ClientID, DatabaseError, TxUID};

use std::collections::HashMap;
use std::sync::RwLock;

/// Database trait needed for Engine.
pub trait Database: Send + Sync {
    fn get_tx(&self, id: &TxUID) -> Result<Option<Transaction>, DatabaseError>;
    fn write_tx(&self, tx: Transaction) -> Result<(), DatabaseError>;
    fn get_client(&self, id: &ClientID) -> Result<Option<Client>, DatabaseError>;
    fn write_client(&self, id: ClientID, client: Client) -> Result<(), DatabaseError>;
    // TODO: return iterator
    fn all_clients(&self) -> Result<Vec<ClientView>, DatabaseError>;
}

/// Uses simple HashMaps to save clients and transactions.
///
/// Uses locks be thread-safe.
pub struct InMemoryDB {
    clients: RwLock<HashMap<ClientID, Client>>,
    transactions: RwLock<HashMap<TxUID, Transaction>>,
}

impl InMemoryDB {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
        }
    }
}

impl Database for InMemoryDB {
    fn get_tx(&self, id: &TxUID) -> Result<Option<Transaction>, DatabaseError> {
        match self.transactions.read() {
            Ok(db) => Ok(db.get(id).cloned()),
            Err(_) => Err(DatabaseError::PoisonLock),
        }
    }

    fn write_tx(&self, tx: Transaction) -> Result<(), DatabaseError> {
        debug_assert!(tx.is_recorded());
        match self.transactions.write() {
            Ok(mut db) => db.insert(tx.uid(), tx),
            Err(_) => return Err(DatabaseError::PoisonLock),
        };
        Ok(())
    }

    fn get_client(&self, id: &ClientID) -> Result<Option<Client>, DatabaseError> {
        match self.clients.read() {
            Ok(db) => Ok(db.get(id).cloned()),
            Err(_) => Err(DatabaseError::PoisonLock),
        }
    }

    fn write_client(&self, id: ClientID, client: Client) -> Result<(), DatabaseError> {
        match self.clients.write() {
            Ok(mut db) => db.insert(id, client),
            Err(_) => return Err(DatabaseError::PoisonLock),
        };
        Ok(())
    }

    fn all_clients(&self) -> Result<Vec<ClientView>, DatabaseError> {
        match self.clients.read() {
            Ok(db) => Ok(db
                .iter()
                .map(|(id, client)| client.get_view(*id))
                .collect::<Vec<ClientView>>()),
            Err(_) => Err(DatabaseError::PoisonLock),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::db::{Database, InMemoryDB};
    use crate::transactions::{DepositTransaction, Transaction, WithdrawalTransaction};
    use rust_decimal_macros::dec;

    #[test]
    fn test_write_get() {
        let db = InMemoryDB::new();
        let tx1 = Transaction::Deposit(DepositTransaction {
            client: 10,
            tx: 1,
            amount: dec!(100.0),
        });
        let tx2 = Transaction::Deposit(DepositTransaction {
            client: 12,
            tx: 5,
            amount: dec!(90.0),
        });
        assert_eq!(db.get_tx(&(10, 1)).unwrap(), None);
        assert_eq!(db.get_tx(&(12, 5)).unwrap(), None);
        assert_eq!(db.get_tx(&(10, 7)).unwrap(), None);
        db.write_tx(tx1.clone()).unwrap();
        assert_eq!(db.get_tx(&(10, 1)).unwrap(), Some(tx1.clone()));
        assert_eq!(db.get_tx(&(12, 5)).unwrap(), None);
        assert_eq!(db.get_tx(&(10, 7)).unwrap(), None);
        db.write_tx(tx2.clone()).unwrap();
        assert_eq!(db.get_tx(&(10, 1)).unwrap(), Some(tx1.clone()));
        assert_eq!(db.get_tx(&(12, 5)).unwrap(), Some(tx2.clone()));
        assert_eq!(db.get_tx(&(10, 7)).unwrap(), None);
        let tx2_new = Transaction::Withdrawal(WithdrawalTransaction {
            client: 12,
            tx: 5,
            amount: dec!(20.0),
        });
        db.write_tx(tx2_new.clone()).unwrap();
        assert_eq!(db.get_tx(&(10, 1)).unwrap(), Some(tx1.clone()));
        assert_eq!(db.get_tx(&(12, 5)).unwrap(), Some(tx2_new.clone()));
        assert_eq!(db.get_tx(&(10, 7)).unwrap(), None);
    }
}
