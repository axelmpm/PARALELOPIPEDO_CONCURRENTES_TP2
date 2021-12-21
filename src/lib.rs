pub mod alglobo;
pub mod leader_election;
pub mod logger;
pub mod message;
pub mod message_body;
pub mod message_kind;
pub mod operation;
pub mod pending_storage;
pub mod processor;
pub mod service;
pub mod service_kind;
pub mod transaction;
pub mod transaction_log_parser;
pub mod transaction_parser;
pub mod transaction_phase;
mod ServiceStream;

use std::env;

use crate::alglobo::Alglobo;
use crate::service::Service;
use std::fs::File;
use std::io;
use std::sync::{mpsc, Arc, Mutex};

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let mode = &args[1];

    match mode.as_ref() {
        "clear" => clear_files(),
        "alglobo" => alglobo(
            args[2]
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR")),
        ),
        "service" => service(args[2].clone()),
        _ => println!("invalid start mode"),
    }
}

fn service(kind: String) {
    let service = Arc::new(Service::new(
        service_kind::parse_kind(kind).unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR")),
    ));
    let (sender, receiver) = mpsc::channel();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        let _ = sender.send(());
    })
    .expect("Error setting Ctrl-C handler");

    service.run(Arc::new(Mutex::new(receiver)));
}

fn alglobo(id: u32) {
    let (sender, receiver) = mpsc::channel();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        let _ = sender.send(());
    })
    .expect("Error setting Ctrl-C handler");

    let mut alglobo = Alglobo::new("localhost".to_string(), 10000, id);
    let exit = alglobo.process(Arc::new(Mutex::new(receiver)));
    if !exit {
        alglobo_retry_mode(alglobo);
    }
}

fn alglobo_retry_mode(mut alglobo: Alglobo) {
    let mut exit = false;
    println!("Welcome to AlGlobo.com! My name is Globby🎈 how can I help you? :)");
    while !exit {
        println!("==========================================");
        println!("Press [F] to see all failed transactions");
        println!("Press [R] to retry a failed transaction");
        println!("Press [X] to exit");

        match read_char_from_stdin().unwrap_or_else(|| panic!("LIB: INTERNAL ERRROR")) {
            'F' => {
                alglobo.show_failed_transactions();
            }
            'R' => {
                println!("input id to retry");
                let id = read_char_from_stdin()
                    .unwrap_or_else(|| panic!("LIB: INTERNAL ERRROR"))
                    .to_digit(10)
                    .unwrap_or_else(|| panic!("LIB: INTERNAL ERRROR"));
                if alglobo.retry(id as i32) {
                    println!("success in retry!");
                } else {
                    println!("id inputed not found");
                }
            }
            'X' => {
                println!("Goodbye!");
                exit = true;
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}

fn read_char_from_stdin() -> Option<char> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
    input = input.to_uppercase();
    return input.chars().next();
}

fn clear_files() {
    File::create("airline.txt").unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
    File::create("hotel.txt").unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
    File::create("bank.txt").unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
    File::create("transaction_log.txt").unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
    File::create("failed_transactions_log.txt").unwrap_or_else(|_| panic!("LIB: INTERNAL ERRROR"));
}
