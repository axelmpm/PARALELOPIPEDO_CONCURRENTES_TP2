pub mod service;
pub mod service_kind;
pub mod alglobo;
pub mod pending_storage;
pub mod message;
pub mod message_kind;
pub mod processor;
pub mod transaction_step;

use std::env;

use crate::service_kind::ServiceKind;
use crate::service::Service;
use crate::alglobo::Alglobo;
use std::sync::Arc;



pub fn run() {

    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    let param = &args[1];

    match param.as_ref() {
        "alglobo" => {
            let host = args[2].to_string();
            let port = args[3].parse::<i32>().unwrap();
            Alglobo::new(host, port).run();
        }
        "service" => {
            
            let service = Arc::new(Service::new(ServiceKind::Hotel));
            let service_clone = service.clone();

            ctrlc::set_handler(move || {
                println!("received Ctrl+C!");
                service_clone.close();
            })
            .expect("Error setting Ctrl-C handler");

            service.run();

        }
        _ => {}
    }
}