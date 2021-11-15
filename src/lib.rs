pub mod service;
pub mod service_kind;
pub mod alglobo;

use std::env;

use crate::service_kind::ServiceKind;
use crate::service::Service;
use crate::alglobo::Alglobo;


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
            Service::new(ServiceKind::Hotel).run();
        }
        _ => {}
    }
}