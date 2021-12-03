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

use std::env;

use crate::service_kind::ServiceKind;
use crate::service::Service;
use crate::alglobo::Alglobo;
use std::sync::{Arc, mpsc, Mutex};
use std::io;



pub fn run() {

    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();
    let param = &args[1];

    match param.as_ref() {
        "alglobo" => alglobo(),
        _ => service(param.clone()),
        // _ => {}
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

    let exit = Alglobo::new("localhost".to_string(), 9000).process(Arc::new(Mutex::new(receiver)));
    
    println!("Welcome to AlGlobo.com! My name is Globby🎈 how can I help you? :)");

    if !exit{
        alglobo_retry_mode();
    }
}

fn alglobo_retry_mode(){
    let mut exit = false;
    let mut invalid_cmd = true;
    while !exit{

        if invalid_cmd{
            println!("Press [F] to see all failed transactions");
            println!("Press [R] to retry a failed transaction");
            println!("Press [X] to exit");
            invalid_cmd = false;
        }

        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer);

        println!("{}",buffer);

        match buffer.to_uppercase().as_ref() {
            "X" => {
                println!("Goodbye!");
                exit = true;
            }
            _ => {
                println!("Unknown command");
                invalid_cmd = true;
            }
        }
        exit = true; // TODO (sacar). Lo pongo porque sino no anda el ctrlc y queda trabado.
    }
}
