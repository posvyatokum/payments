# Execution
```bash
cargo run -- <input filename> > <output filename>
```
If input file is not specified, program reads from stdin.

# Architecture
## Assumptions
Inputs is given in a correct format:
- no unknown transaction types
- trailing comas for `Dispute`, `Resolve`, and `Chargeback` transactions
- header line

Client ids fit in `u16`.  
Transaction ids fit in `u64`.  
Transaction ids are unique to the client, but not between clients.
## Structure
`types.rs` -- basic data types and errors used throughout the code.  
`client.rs` -- structures related to Client creation and update.  
`transaction.rs` -- structures related to different types of Transactions.  
`db.rs` -- definition of the `Database` trait and implementation of `InMemoryDB`.  
`engine.rs` -- business logic.  
`flow.rs` -- full flow.
## Complexity
All clients are stored in memory.  
Additionally, all the deposit and withdrawal transactions are stored as well.  
Every transaction is processed in constant time.  
Output is processed in linear time with linear memory overhead.
## Transaction
Internal `Transaction` enum.
### TransactionView
Structure that represent transaction input,
that can be later converted into the internal transaction structure.
## Client
Has all the client data without the id,
as it is not needed within the current implementation,
except for the output.
### ClientView
Structure to output client data in a specific format.
## Database
Provides thread-safe access to internal data.
TODO: abstract stored data types.
### InMemoryDB
Uses HashMap structures to store clients and transactions.
Guards them with a lock for thread-safety
(in case of future multi-thread developments).
## Amount
To perform operations with precision `rust_decimal::Decimal` is used.

# Testing
## Unit tests
`engine` module has a unit test for every type of transaction.  
`db` module has test to check `get/set` methods with new data and overwrites.
## Integration tests
`flow` module has several full flow tests
that check output against predetermined correct output
in an order-agnostic way.