pub mod service;
pub mod service_kind;
pub mod alglobo;
pub mod pending_storage;
pub mod message;
pub mod message_kind;
pub mod processor;
pub mod message_body;
pub mod logger;

use std::env;

use crate::service_kind::ServiceKind;
use crate::service::Service;
use crate::alglobo::Alglobo;
use std::sync::{Arc, mpsc, Mutex};



pub fn run() {

    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();
    let param = &args[1];

    match param.as_ref() {
        "alglobo" => {
            //let host = args[2].to_string();
            //let port = args[3].parse::<i32>().unwrap();

            let (sender, receiver) = mpsc::channel();

            ctrlc::set_handler(move || {
                println!("received Ctrl+C!");
                let _ = sender.send(());
            })
            .expect("Error setting Ctrl-C handler");

            Alglobo::new("localhost".to_string(), 9000).run(Arc::new(Mutex::new(receiver)));
        }
        "service" => {
            
            let service = Arc::new(Service::new(ServiceKind::Hotel));
            let (sender, receiver) = mpsc::channel();

            ctrlc::set_handler(move || {
                println!("received Ctrl+C!");
                let _ = sender.send(());
            })
            .expect("Error setting Ctrl-C handler");

            service.run(Arc::new(Mutex::new(receiver)));

        }
        _ => {}
    }
}