pub type ClientID = u16;
pub type TxID = u64;
pub type TxUID = (ClientID, TxID);
pub type Amount = rust_decimal::Decimal;

#[derive(Debug)]
pub enum DatabaseError {
    PoisonLock,
}

#[derive(Debug)]
pub enum ClientError {
    DatabaseError(DatabaseError),
}

#[derive(Debug)]
pub enum EngineError {
    ClientError(ClientError),
    DatabaseError(DatabaseError),
}

impl From<DatabaseError> for ClientError {
    fn from(e: DatabaseError) -> Self {
        ClientError::DatabaseError(e)
    }
}

impl From<ClientError> for EngineError {
    fn from(e: ClientError) -> Self {
        EngineError::ClientError(e)
    }
}

impl From<DatabaseError> for EngineError {
    fn from(e: DatabaseError) -> Self {
        EngineError::DatabaseError(e)
    }
}
