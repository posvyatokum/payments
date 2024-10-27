use crate::types::{Amount, ClientID, TxID, TxUID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransactionView {
    #[serde(rename = "type")]
    pub type_str: String,
    pub client: ClientID,
    pub tx: TxID,
    pub amount: Option<Amount>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Transaction {
    Deposit(DepositTransaction),
    Withdrawal(WithdrawalTransaction),
    Dispute(DisputeTransaction),
    Resolve(ResolveTransaction),
    Chargeback(ChargebackTransaction),
}

impl TryFrom<TransactionView> for Transaction {
    type Error = &'static str;

    fn try_from(tx: TransactionView) -> Result<Self, Self::Error> {
        match tx.type_str.as_str() {
            "deposit" => Ok(Transaction::Deposit(DepositTransaction {
                client: tx.client,
                tx: tx.tx,
                amount: tx
                    .amount
                    .expect("No amount provided for the Deposit transaction."),
            })),
            "withdrawal" => Ok(Transaction::Withdrawal(WithdrawalTransaction {
                client: tx.client,
                tx: tx.tx,
                amount: tx
                    .amount
                    .expect("No amount provided for the Withdrawal transaction."),
            })),
            "dispute" => Ok(Transaction::Dispute(DisputeTransaction {
                client: tx.client,
                tx: tx.tx,
            })),

            "resolve" => Ok(Transaction::Resolve(ResolveTransaction {
                client: tx.client,
                tx: tx.tx,
            })),

            "chargeback" => Ok(Transaction::Chargeback(ChargebackTransaction {
                client: tx.client,
                tx: tx.tx,
            })),
            _ => Err("unexpected transaction type"),
        }
    }
}

impl Transaction {
    pub fn client(&self) -> ClientID {
        match self {
            Transaction::Deposit(tx) => tx.client,
            Transaction::Withdrawal(tx) => tx.client,
            Transaction::Dispute(tx) => tx.client,
            Transaction::Resolve(tx) => tx.client,
            Transaction::Chargeback(tx) => tx.client,
        }
    }

    pub fn id(&self) -> TxID {
        match self {
            Transaction::Deposit(tx) => tx.tx,
            Transaction::Withdrawal(tx) => tx.tx,
            Transaction::Dispute(tx) => tx.tx,
            Transaction::Resolve(tx) => tx.tx,
            Transaction::Chargeback(tx) => tx.tx,
        }
    }

    pub fn uid(&self) -> TxUID {
        (self.client(), self.id())
    }

    pub fn is_recorded(&self) -> bool {
        matches!(self, Transaction::Deposit(_) | Transaction::Withdrawal(_))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DepositTransaction {
    pub client: ClientID,
    pub tx: TxID,
    pub amount: Amount,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WithdrawalTransaction {
    pub client: ClientID,
    pub tx: TxID,
    pub amount: Amount,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DisputeTransaction {
    pub client: ClientID,
    pub tx: TxID,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ResolveTransaction {
    pub client: ClientID,
    pub tx: TxID,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChargebackTransaction {
    pub client: ClientID,
    pub tx: TxID,
}
