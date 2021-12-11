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
    let exit = false;
    if !exit{
        alglobo_retry_mode(alglobo);
    }
}

fn alglobo_retry_mode(alglobo: Alglobo){
    let mut exit = false;
    let mut invalid_cmd = true;
    println!("Welcome to AlGlobo.com! My name is Globby🎈 how can I help you? :)");
    while !exit{

        if invalid_cmd{
            println!("Press [F] to see all failed transactions");
            println!("Press [R] to retry a failed transaction");
            println!("Press [X] to exit");
            invalid_cmd = false;
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.to_uppercase();

        match input.chars().next().unwrap() {
            'F' => {
                println!("tocaste la F!");
                alglobo.show_failed_transactions();
            },
            'R' => {
                println!("tocaste la R!");
            },
            'X' => {
                println!("Goodbye!");
                exit = true;
            }
            _ => {
                println!("Unknown command");
                invalid_cmd = true;
            }
        }
    }
}
