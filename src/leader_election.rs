use std::convert::TryInto;
use std::mem::size_of;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const N_NODES: u32 = 5;
const TIME: u64 = 1500;
const TIMEOUT: Option<Duration> = Some(Duration::from_millis(TIME));
const NON_LEADER_ID: u32 = u32::MAX;

pub struct LeaderElection {
    id: u32,
    host: String,
    port: u32,
    sock: UdpSocket,
    ping_sock: UdpSocket,
    current_leader: Arc<(Mutex<Option<u32>>, Condvar)>,
    electing: Arc<Mutex<bool>>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    leader_changed: Arc<(Mutex<bool>, Condvar)>,
    finished: Arc<Mutex<bool>>,
}

fn to_pingaddr(host: String, port: u32, id: u32) -> String {
    format!("{}:{}", host, port - id - 1)
}
fn to_workaddr(host: String, port: u32, id: u32) -> String {
    format!("{}:{}", host, port + id)
}

impl Clone for LeaderElection {
    fn clone(&self) -> Self {
        LeaderElection {
            id: self.id,
            host: self.host.clone(),
            port: self.port,
            sock: self.sock.try_clone().unwrap(),
            ping_sock: self.ping_sock.try_clone().unwrap(),
            current_leader: self.current_leader.clone(),
            electing: self.electing.clone(),
            got_ok: self.got_ok.clone(),
            leader_changed: self.leader_changed.clone(),
            finished: self.finished.clone(),
        }
    }
}

impl LeaderElection {
    pub fn new(host: String, port: u32, id: u32) -> LeaderElection {
        let addr = to_workaddr(host.clone(), port, id);
        let addr2 = to_pingaddr(host.clone(), port, id);
        let sock = UdpSocket::bind(addr).expect("could not create socket");
        let ping_sock = UdpSocket::bind(addr2).expect("could not create socket");
        sock.set_read_timeout(TIMEOUT).expect("could not set timeout");

        let leader = LeaderElection {
            id,
            host,
            port,
            sock,
            ping_sock,
            current_leader: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            electing: Arc::new(Mutex::new(false)),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            leader_changed: Arc::new((Mutex::new(false), Condvar::new())),
            finished: Arc::new(Mutex::new(false)),
        };
        let leader_clone = leader.clone();

        thread::spawn(move || loop {
            //answer messages
            let mut buf = [0; size_of::<u32>() + 1];
            if let Ok((_size, from)) = leader_clone.sock.recv_from(&mut buf) {
                leader_clone.process_message(
                    buf[0],
                    u32::from_le_bytes(buf[1..].try_into().unwrap()),
                    from,
                );
            }
            if leader_clone.is_done() {
                break;
            }
        });

        leader.elect_new_leader();

        leader
    }

    pub fn is_done(&self) -> bool {
        *self.finished.lock().unwrap()
    }

    fn id_to_msg(&self, header: u8) -> Vec<u8> {
        let mut msg = vec![header];
        msg.extend_from_slice(&self.id.to_le_bytes());
        msg
    }

    pub fn elect_new_leader(&self) {
        let mut electing = self.electing.lock().unwrap();
        if *electing {
            return;
        }
        *electing = true;
        drop(electing);
        *self.current_leader.0.lock().unwrap() = None;
        *self.got_ok.0.lock().unwrap() = false;

        println!("CALLING ELECTION {}", self.id);
        self.send_election();

        let got_ok = self.got_ok.1.wait_timeout_while(
            self.got_ok.0.lock().unwrap(),
            TIMEOUT.unwrap(),
            |got_it| !*got_it,
        );
        if *got_ok.unwrap().0 {
            //if recv ok then wait to recv new leader id
            println!("GOT OK  {}", self.id);
            self.current_leader
                .1
                .wait_while(self.current_leader.0.lock().unwrap(), |leader_id| {
                    leader_id.is_none()
                }).expect("could not get leader id");
        } else {
            //if no ok arrived then proclaim myself as leader
            println!("IM LEADER {}", self.id);
            self.make_leader(self.id);
            println!("PROCLAIMING LEADER {}", self.id);
            self.send_leader_proclamation();
        }
        *self.electing.lock().unwrap() = false;
        println!("FINISHED ELECTION {}", self.id);
    }

    pub fn ping_control(&self) {
        println!("SETTING UP PINGS");
        while !self.is_done() {
            let lid = self.get_leader_id();
            let mut buf = [0; size_of::<u32>() + 1];

            if lid == self.id {
                self.ping_sock.set_read_timeout(None).expect("could not set timeout");
            } else {
                self.ping_sock.set_read_timeout(TIMEOUT).expect("could not set timeout");
                self.ping_sock.send_to(
                    &self.id_to_msg(b'P'),
                    to_pingaddr(self.host.clone(), self.port, lid),
                ).expect("could not send");
                thread::sleep(Duration::from_millis(1000));
            }

            match self.ping_sock.recv_from(&mut buf) {
                Ok((_size, from)) => {
                    self.process_message(
                        buf[0],
                        u32::from_le_bytes(buf[1..].try_into().unwrap()),
                        from,
                    );
                }
                Err(_e) => {
                    //timeout or error
                    self.elect_new_leader();
                }
            }
        }
        println!("IM DONE WITH PINGS ");
    }

    pub fn close(&self, tell_others: bool) {
        if tell_others {
            self.send_close();
        }
        *self.finished.lock().unwrap() = true;
        self.sock.send_to(
            &self.id_to_msg(b'X'),
            to_pingaddr(self.host.clone(), self.port, self.id),
        ).expect("could not send");
    }

    pub fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    pub fn wait_until_leader_changes(&self) {
        self.leader_changed
            .1
            .wait_while(self.leader_changed.0.lock().unwrap(), |changed| !*changed)
            .expect("could not get changed signal");
        *self.leader_changed.0.lock().unwrap() = false;
    }

    fn get_leader_id(&self) -> u32 {
        self.current_leader
            .1
            .wait_while(self.current_leader.0.lock().unwrap(), |leader_id| {
                leader_id.is_none()
            })
            .unwrap()
            .unwrap()
    }

    fn process_message(&self, msg: u8, id_from: u32, src: SocketAddr) {
        match msg {
            b'P' => {
                //println!("GOT PING FROM {}", id_from);
                self.ping_sock.send_to(&self.id_to_msg(b'Z'), src).expect("could not send");
            }
            b'E' => {
                //call to elections
                println!("GOT ELECTION FROM {}", id_from);
                if self.id <= id_from {
                    return;
                }
                println!("SEND OK TO {} FROM {}", id_from, self.id);
                self.sock.send_to(&self.id_to_msg(b'O'), src).expect("could not send");
                let clone = self.clone();
                thread::spawn(move || clone.elect_new_leader());
            }
            b'L' => {
                println!("GOT {} AS LEADER", id_from);
                self.make_leader(id_from);
                let addr2 = to_pingaddr(self.host.clone(), self.port, self.id);//wake up
                self.sock.send_to(&self.id_to_msg(b'Z'), addr2).expect("could not send");
            } //update leader condvar and leader id value//new leader
            b'O' => {
                *self.got_ok.0.lock().unwrap() = true;
                self.got_ok.1.notify_all();
            } //update OK condvar
            b'X' => {
                //close signal recv
                println!("GOT CLOSE FROM {}", id_from);
                *self.finished.lock().unwrap() = true;
                self.make_leader(NON_LEADER_ID);
            }
            _ => {}
        }
    }

    fn make_leader(&self, id_from: u32) {
        *self.current_leader.0.lock().unwrap() = Some(id_from);
        self.current_leader.1.notify_all();
        *self.leader_changed.0.lock().unwrap() = true;
        self.leader_changed.1.notify_all();
    }

    fn send_leader_proclamation(&self) {
        for i in 0..N_NODES {
            //broadcast to all
            if i != self.id {
                let addr = to_workaddr(self.host.clone(), self.port, i);
                self.sock.send_to(&self.id_to_msg(b'L'), addr).expect("could not send");
            }
        }
    }

    fn send_close(&self) {
        for i in 0..N_NODES {
            //broadcast to all
            if i != self.id {
                let addr = to_workaddr(self.host.clone(), self.port, i);
                let addr2 = to_pingaddr(self.host.clone(), self.port, i);
                self.sock.send_to(&self.id_to_msg(b'X'), addr).expect("could not send");
                self.sock.send_to(&self.id_to_msg(b'X'), addr2).expect("could not send");
            }
        }
    }

    fn send_election(&self) {
        for i in (self.id + 1)..N_NODES {
            //broadcast to bigger numbers
            let addr = to_workaddr(self.host.clone(), self.port, i);
            self.sock.send_to(&self.id_to_msg(b'E'), addr).expect("could not send");
        }
    }
}
