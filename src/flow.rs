use crate::client::ClientView;
use crate::engine::Engine;
use crate::transactions::{Transaction, TransactionView};
use csv::{ReaderBuilder, WriterBuilder};

pub fn process_csv_transactions<R: std::io::Read>(input: R) -> Vec<ClientView> {
    let engine = Engine::new();

    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(input);

    for record in reader.deserialize() {
        let tx: TransactionView = record.unwrap();
        let tx = Transaction::try_from(tx).unwrap();
        engine.process_transaction(&tx).unwrap();
    }

    engine.get_all_clients().unwrap()
}

pub fn output_csv_clients<W: std::io::Write>(clients: Vec<ClientView>, output: W) {
    let mut wtr = WriterBuilder::new()
        .flexible(true)
        .has_headers(true)
        .from_writer(output);

    for client in clients {
        wtr.serialize(client).unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::client::ClientView;
    use crate::flow::process_csv_transactions;
    use csv::ReaderBuilder;
    use std::collections::HashSet;

    fn test_sample(input: String, correct_output: String) {
        let result_set =
            HashSet::from_iter(process_csv_transactions(&mut input.as_bytes()).into_iter());

        let mut correct_set = HashSet::new();
        let mut correct_buff = correct_output.as_bytes();

        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(&mut correct_buff);

        for record in reader.deserialize() {
            let client: ClientView = record.unwrap();
            correct_set.insert(client);
        }

        assert_eq!(result_set, correct_set)
    }

    #[test]
    fn test_flow1() {
        let _ = env_logger::try_init();
        test_sample(
            "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0"
                .to_string(),
            "\
client, available, held, total, locked
1, 1.5, 0.0, 1.5, false
2, 2.0, 0.0, 2.0, false"
                .to_string(),
        )
    }

    #[test]
    fn test_flow2() {
        let _ = env_logger::try_init();
        test_sample(
            "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 1, 2, 2.3412
withdrawal, 1, 3, 5.5
withdrawal, 1, 4, 3.0
dispute, 1, 2,
dispute, 1, 1,
withdrawal, 1, 5, 0.0001
deposit, 1, 6, 1.0
chargeback, 1, 2,
deposit, 1, 7, 10.0
withdrawal, 1, 8, 0.1"
                .to_string(),
            "\
client, available, held, total, locked
1, -2.0, 1.0, -1.0, true"
                .to_string(),
        )
    }

    #[test]
    fn test_flow3() {
        let _ = env_logger::try_init();
        test_sample(
            "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 1, 2, 2.3412
withdrawal, 1, 3, 5.5
withdrawal, 1, 4, 3.0
dispute, 1, 2,
dispute, 1, 1,
withdrawal, 1, 5, 0.0001
deposit, 1, 6, 1.0
chargeback, 1, 1,
deposit, 1, 7, 10.0
withdrawal, 1, 8, 0.1"
                .to_string(),
            "\
client, available, held, total, locked
1, -2.0, 2.3412, 0.3412, true"
                .to_string(),
        )
    }
}
