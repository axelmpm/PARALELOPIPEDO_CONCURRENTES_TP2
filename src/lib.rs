pub mod service;
pub mod service_kind;
pub mod alglobo;
pub mod pending_storage;
pub mod message;
pub mod message_kind;
pub mod processor;
pub mod message_body;
pub mod logger;
pub mod transaction_parser;
pub mod operation;
pub mod transaction;
pub mod leader_election;

use std::env;

use crate::service::Service;
use crate::alglobo::Alglobo;
use std::sync::{Arc, mpsc, Mutex};
use std::io;


pub fn run() {

    let args: Vec<String> = env::args().collect();
    let mode = &args[1];

    match mode.as_ref() {
        "alglobo" => alglobo(),
        "service" => service(args[2].clone()),
        _ => println!("invalid start mode")
    }
}

fn service(kind: String){
            
    let service = Arc::new(Service::new(service_kind::parse_kind(kind).unwrap()));
    let (sender, receiver) = mpsc::channel();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        let _ = sender.send(());
    })
    .expect("Error setting Ctrl-C handler");

    service.run(Arc::new(Mutex::new(receiver)));

}

fn alglobo(){
    //let host = args[2].to_string();
    //let port = args[3].parse::<i32>().unwrap();

    let (sender, receiver) = mpsc::channel();
    
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        let _ = sender.send(());
    })
    .expect("Error setting Ctrl-C handler");

    let mut alglobo = Alglobo::new("localhost".to_string(), 9000);
    let exit = alglobo.process(Arc::new(Mutex::new(receiver)));
    if !exit{
        alglobo_retry_mode(alglobo);
    }
}

fn alglobo_retry_mode(mut alglobo: Alglobo){
    let mut exit = false;
    println!("Welcome to AlGlobo.com! My name is Globby🎈 how can I help you? :)");
    while !exit{

        println!("");
        println!("==========================================");
        println!("Press [F] to see all failed transactions");
        println!("Press [R] to retry a failed transaction");
        println!("Press [X] to exit");
        
        match read_char_from_stdin().unwrap() {
            'F' => {
                alglobo.show_failed_transactions();
            },
            'R' => {
                println!("input id to retry");
                let id = read_char_from_stdin().unwrap().to_digit(10).unwrap();
                if alglobo.retry(id as i32) {
                    println!("success in retry!");
                } else {
                    println!("id inputed not found");
                }
            },
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


fn read_char_from_stdin() -> Option<char>{
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input = input.to_uppercase();
    return input.chars().next();
}