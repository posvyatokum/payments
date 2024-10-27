mod client;
mod db;
mod engine;
mod flow;
mod transactions;
mod types;

use log::info;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1) {
        None => {
            info!("Reading data from stdin.");
            flow::output_csv_clients(
                flow::process_csv_transactions(std::io::stdin()),
                std::io::stdout(),
            );
        }
        Some(filename) => {
            info!("Reading data from {filename}");
            let reader = std::io::BufReader::new(std::fs::File::open(filename).unwrap());
            flow::output_csv_clients(flow::process_csv_transactions(reader), std::io::stdout());
        }
    }
}