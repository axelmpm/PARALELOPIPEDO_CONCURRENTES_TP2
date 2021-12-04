use std::net::UdpSocket;
use std::sync::{Mutex, Arc, Condvar};
use std::time::Duration;
use std::thread;

pub const N_NODES: u32 = 1;
const BASE_PORT: u32 = 30000;
const TIME: u64 = 1500;
const TIMEOUT: Some<Duration> = Some(Duration::from_millis(TIMEOUT));

pub struct ScrumMaster{
    id: u32,
    sock: UdpSocket,
    current_leader: Arc<Mutex<u32>>,
    electing: Arc<Mutex<bool>>
}

impl LeaderElection {
    fn new(id: u32) -> LeaderElection {
        addr = format!("127.0.0.1:{}",BASE_PORT + id);
        sock = UdpSocket::bind(addr).expect("could not create socket");
        sock.set_read_timeout(TIMEOUT);
        return LeaderElection{
            id,
            sock,
            current_leader: Arc::new(Mutex::new(id)),
            electing: Arc::new(Mutex::new(false)),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            sock: self.sock.try_clone().unwrap(),
            current_leader: self.current_leader.clone(),
            electing: self.electing.clone(),
            got_ok: self.got_ok.clone()
        }
    }

    pub fn elect_new_leader(&self){
        let electing = self.electing.lock().unwrap();
        if electing {
            return;
        }
        *electing = true;
        drop(electing);

        self.send_election();

        let got_ok = self.got_ok.1.wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT, |got_it| !*got_it );
        if *got_ok.unwrap().0 {
            //if recv ok then wait to recv new leader id
        } else {
            //if no ok arrived then proclaim myself as leader
            self.send_leader_proclamation();
        }



    }

    pub fn answer(&self){
        //self.set_nonblocking(true).expect("could not set non-blocking");
        let mut buf = String::new();

        match socket.recv_from(&mut buf){
            Ok((amt, src)) => self.process_message(amt, buf, src),
            Err(ref e) if e.kind() == WouldBlock => {
                //check controlc
            },

            Err(e) => panic!("encountered IO error: {}", e),
        }

    }

    fn process_message(&self, amt: usize, buf: String, src: String) {
        match buf.as_ref() {
            "P" => {self.sock.send_to("P".as_ref(),src);}, // ping
            "E" => {//call to elections
                self.sock.send_to("O".as_ref(), src); // send ok to caller
                let clone = self.clone();
                thread::spawn(move ||
                    clone.elect_new_leader() );
            },
            "L" => {},//update leader condvar and leader id value//new leader
            "O" => {
                *self.got_ok.0.lock().unwrap() = true;
                self.got_ok.1.notify_all();
            },//update OK condvar
            _ => {}
        }
    }


    fn ping_pong_leader(&self) -> bool{
        self.sock.send_to("P".as_ref(), addr = format!("127.0.0.1:{}", BASE_PORT + self.current_leader));
        let mut buf = String::new();
        let (amt, src) = socket.recv_from(&mut buf).expect("could not recv udp");

        amt != 0 && buf != "E"
    }

    fn send_leader_proclamation(&self){//todo change strings into structs
        for i in 0..N_NODES { //broadcast to all
            self.sock.send_to(format!("L-{}", self.id).as_ref(), addr = format!("127.0.0.1:{}", BASE_PORT + i));
        }
    }

    fn send_election(&self){
        for i in (self.id+1)..N_NODES {//broadcast to bigger numbers
                self.sock.send_to("E".as_ref(), addr = format!("127.0.0.1:{}", BASE_PORT + i));
            }
        }
    }
}
