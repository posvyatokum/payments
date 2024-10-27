use crate::transactions::{DepositTransaction, WithdrawalTransaction};
use crate::types::{Amount, ClientError, ClientID, TxID};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ClientStatus {
    #[default]
    Live,
    Frozen,
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Client {
    pub available: Amount,
    pub held: Amount,
    pub status: ClientStatus,
    /// Set of all disputed transactions without resolution for this Client
    pub disputes: HashSet<TxID>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct ClientView {
    pub client: ClientID,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl Client {
    pub fn is_frozen(&self) -> bool {
        self.status == ClientStatus::Frozen
    }

    pub fn process_deposit(&mut self, tx: &DepositTransaction) -> Result<(), ClientError> {
        self.available += tx.amount;
        Ok(())
    }

    pub fn process_withdrawal(&mut self, tx: &WithdrawalTransaction) -> Result<(), ClientError> {
        if self.available < tx.amount {
            log::warn!(target: "client", "Insufficient funds for withdrawal {tx:?}. {self:?}");
            return Ok(());
        }
        self.available -= tx.amount;
        Ok(())
    }

    pub fn process_dispute(&mut self, tx: &DepositTransaction) -> Result<(), ClientError> {
        self.available -= tx.amount;
        self.held += tx.amount;
        self.disputes.insert(tx.tx);
        Ok(())
    }

    pub fn process_chargeback(&mut self, tx: &DepositTransaction) -> Result<(), ClientError> {
        self.held -= tx.amount;
        self.disputes.remove(&tx.tx);
        self.status = ClientStatus::Frozen;
        Ok(())
    }

    pub fn process_resolve(&mut self, tx: &DepositTransaction) -> Result<(), ClientError> {
        self.held -= tx.amount;
        self.available += tx.amount;
        self.disputes.remove(&tx.tx);
        Ok(())
    }

    pub fn get_view(&self, id: ClientID) -> ClientView {
        ClientView {
            client: id,
            available: self.available,
            held: self.held,
            total: self.available + self.held,
            locked: self.is_frozen(),
        }
    }
}
