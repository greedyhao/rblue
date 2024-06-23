use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use rblue_core::{hci::*, BDAddr, BtCmd};

struct SimBB {
    channels: HashMap<BDAddr, Sender<Vec<u8>>>,
    phy: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}

impl SimBB {
    fn new() -> Self {
        SimBB {
            channels: HashMap::new(),
            phy: mpsc::channel(),
        }
    }

    fn get_phy(&self) -> Sender<Vec<u8>> {
        return self.phy.0.clone();
    }

    fn insert(&mut self, addr: BDAddr, channel: Sender<Vec<u8>>) {
        self.channels.insert(addr, channel);
    }

    fn run(&mut self) {
        if let Ok(data) = self.phy.1.try_recv() {
            let addr = &data[0..6];
            let addr = BDAddr::try_from(addr).unwrap();
            if let Some(tx) = self.channels.get_mut(&addr) {
                tx.send((&data[6..]).to_vec()).unwrap();
            }
        }
    }
}

pub trait HciSender {
    fn send_packet(&mut self, packet: u8, data: Vec<u8>);
}

impl HciSender for Hci<Sender<Vec<u8>>> {
    fn send_packet(&mut self, packet: u8, data: Vec<u8>) {
        if let Some(sender) = self.get_sender() {
            let mut tmp = vec![packet];
            tmp.extend(data);
            sender.send(tmp).unwrap();
        }
    }
}

fn create_new_hci(bb: &mut SimBB, bd_addr: BDAddr) -> Sender<BtCmd> {
    let (app_tx, app_rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    bb.insert(bd_addr, tx);
    let phy_tx = bb.get_phy().clone();

    let mut hci: Hci<Sender<Vec<u8>>> = Hci::new(bd_addr);
    hci.set_sender(phy_tx);
    hci.set_send_packet(|this, packet, data| {
        if let Some(tx) = this.get_sender() {
            let mut tmp = vec![packet];
            tmp.extend(this.get_bd_addr());
            tmp.extend(data);
            tx.send(tmp).unwrap();
        }
    });

    thread::spawn(move || {
        let mut data = None;
        let mut cmd: Option<BtCmd> = None;
        loop {
            if cmd.is_none() {
                cmd = Some(app_rx.recv().unwrap());
            }
            cmd.unwrap().exec();

            println!("{:?} start", hci.get_bd_addr());
            loop {
                if data == None {
                    data = match rx.try_recv() {
                        Ok(data) => Some(data),
                        Err(_) => None,
                    }
                }

                // process data
                if let Some(data) = data {
                    if data.len() > 0 {
                        let packet = data[0];
                        let data = (&data[1..]).to_owned();
                        match packet {
                            1 => hci.recv_ce_data(data),
                            2 => hci.recv_acl_data(data),
                            _ => panic!("error packet"),
                        }
                    }
                }

                // do something here

                // receive data and process in next loop
                data = match rx.recv_timeout(Duration::from_millis(5)) {
                    Ok(data) => Some(data),
                    Err(_) => None,
                };

                // check if there is a command
                cmd = app_rx.try_recv().map_or(None, |cmd| Some(cmd));
                if cmd.is_some() {
                    break;
                }
            }
        }
    });
    return app_tx;
}

fn main() {
    env_logger::init();
    log::debug!("debug log test");

    let mut sim_bb = SimBB::new();

    let addr1 = [1, 0, 0, 0, 0, 0];
    let addr2 = [2, 0, 0, 0, 0, 0];
    let app1 = create_new_hci(&mut sim_bb, addr1);
    let app2 = create_new_hci(&mut sim_bb, addr2);

    // just test
    app1.send(BtCmd::Test).unwrap();
    app2.send(BtCmd::Test).unwrap();

    let phy_tx = sim_bb.get_phy().clone();
    let bb: thread::JoinHandle<_> = thread::spawn(move || loop {
        sim_bb.run();
    });

    // tx test
    thread::spawn(move || {
        let mut tmp = Vec::from(addr1);
        tmp.extend([1, 11, 12, 13, 14]);
        phy_tx.send(tmp).unwrap();
    });

    // pend
    bb.join().unwrap();
}
