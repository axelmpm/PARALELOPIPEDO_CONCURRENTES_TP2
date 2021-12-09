use std::net::UdpSocket;
use std::sync::{Mutex, Arc, Condvar, mpsc};
use std::time::Duration;
use std::thread;
use std::convert::TryInto;
use std::mem::size_of;
use std::net::SocketAddr;
use std::sync::mpsc::{Sender, Receiver};

pub const N_NODES: u32 = 1;
const TIME: u64 = 1500;
const TIMEOUT: Option<Duration> = Some(Duration::from_millis(TIME));

pub struct LeaderElection{
    id: u32,
    sock: UdpSocket,
    current_leader: Arc<(Mutex<Option<u32>>, Condvar)>,
    electing: Arc<Mutex<bool>>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    leader_changed: Arc<(Mutex<bool>, Condvar)>,
    finish_snd: Arc<Mutex<Sender<bool>>>,
    finish_rcv: Arc<Mutex<Receiver<bool>>>
}

impl LeaderElection {
    pub fn new(id: u32) -> LeaderElection {
        let addr = format!("127.0.0.1:{}", id);
        let sock = UdpSocket::bind(addr).expect("could not create socket");
        sock.set_read_timeout(TIMEOUT);
        let (sender, receiver) = mpsc::channel();

        return LeaderElection{
            id,
            sock,
            current_leader: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            electing: Arc::new(Mutex::new(false)),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            leader_changed: Arc::new((Mutex::new(false), Condvar::new())),
            finish_snd: Arc::new(Mutex::new(sender)),
            finish_rcv: Arc::new(Mutex::new(receiver))
        }
    }

    pub fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            sock: self.sock.try_clone().unwrap(),
            current_leader: self.current_leader.clone(),
            electing: self.electing.clone(),
            got_ok: self.got_ok.clone(),
            leader_changed: self.leader_changed.clone(),
            finish_snd: self.finish_snd.clone(),
            finish_rcv: self.finish_rcv.clone()
        }
    }

    pub fn elect_new_leader(&self){
        let mut electing = self.electing.lock().unwrap();
        if *electing {
            return;
        }
        *electing = true;
        drop(electing);

        self.send_election();

        let got_ok = self.got_ok.1.wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT.unwrap(), |got_it| !*got_it );
        if *got_ok.unwrap().0 {
            //if recv ok then wait to recv new leader id
            self.current_leader.1.wait_while(self.current_leader.0.lock().unwrap(), |leader_id| leader_id.is_none() ); //TODO meter algun TIMEOUT grande (capaz?)
        } else {
            //if no ok arrived then proclaim myself as leader
            self.send_leader_proclamation();
        }
    }

    pub fn work(&self){

        loop {
            let lid = self.get_leader_id();
            if lid == self.id {
                self.sock.set_read_timeout(None);
            } else {
                let addr = format!("127.0.0.1:{}", lid);
                self.sock.set_read_timeout(TIMEOUT);
                self.sock.send_to(format!("P{}", self.id).as_ref(), addr);
            }

            let mut buf = [0; size_of::<u32>() + 1];
            if let Ok((size, from)) = self.sock.recv_from(&mut buf) {
                self.process_message(buf[0], u32::from_le_bytes(buf[1..].try_into().unwrap()), from);
            } else {
                //timeout
                self.elect_new_leader();
            }

            if self.finish_rcv.lock().expect("failed to lock").try_recv().is_ok() {
                break;
            }
        }
    }

    pub fn close(&self){
        self.send_close();
        self.finish_snd.lock().expect("could not lock").send(true);
    }

    pub fn am_i_leader(&self)->bool {
        self.get_leader_id() == self.id
    }

    pub fn wait_until_leader_changes(&self){
         self.leader_changed.1.wait_timeout_while(self.leader_changed.0.lock().unwrap(), TIMEOUT.unwrap(), |changed| !*changed );
        *self.leader_changed.0.lock().unwrap() = false;
    }

    fn get_leader_id(&self) -> u32 {
        self.current_leader.1.wait_while(self.current_leader.0.lock().unwrap(), |leader_id| leader_id.is_none()).unwrap().unwrap()
    }

    fn process_message(&self, msg: u8, id_from:u32, src: SocketAddr) {
        match msg {
            b'P' => {
                self.sock.send_to(format!("R{}", self.id).as_ref(),src);// response to ping
            },
            b'E' => {//call to elections
                if self.id >= id_from { return }

                self.sock.send_to(format!("O{}", self.id).as_ref(), src); // send ok to caller
                let clone = self.clone();
                thread::spawn(move ||
                    clone.elect_new_leader() );
            },
            b'L' => {
                self.make_leader(id_from);
            },//update leader condvar and leader id value//new leader
            b'O' => {
                *self.got_ok.0.lock().unwrap() = true;
                self.got_ok.1.notify_all();
            },//update OK condvar
            b'X' => {
                //close signal recv
                self.finish_snd.lock().expect("could not lock").send(true);
                self.make_leader(id_from);
            }
            _ => {}
        }
    }

    fn make_leader(&self, id_from: u32){
        *self.current_leader.0.lock().unwrap() = Some(id_from);
        self.current_leader.1.notify_all();
        *self.leader_changed.0.lock().unwrap() = true;
        self.leader_changed.1.notify_all();
    }

    fn send_leader_proclamation(&self){
        self.send_all_but_self("L".parse().unwrap());
    }

    fn send_close(&self){
        self.send_all_but_self("X".parse().unwrap());
    }

    fn send_election(&self){
        for i in (self.id+1)..N_NODES {//broadcast to bigger numbers
            let addr = format!("127.0.0.1:{}", i);
            self.sock.send_to(format!("E{}", self.id).as_ref(), addr);
        }
    }

    fn send_all_but_self(&self, str: String){
        for i in 0..N_NODES { //broadcast to all
            if i != self.id {
                let addr = format!("127.0.0.1:{}", i);
                self.sock.send_to(format!("{}{}", str, self.id).as_ref(), addr);
            }
        }
    }
}
