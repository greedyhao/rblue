use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use log::info;
use rblue_core::{
    baseband::{self, ble},
    host::{self, hci::*},
    BDAddr,
};

struct SimPhy {
    link: HashMap<u8, Sender<Vec<u8>>>,
    phy: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    cnt: u8, //TODO
}

impl SimPhy {
    fn new() -> Self {
        SimPhy {
            link: HashMap::new(),
            phy: mpsc::channel(),
            cnt: 0,
        }
    }

    fn get_phy(&self) -> Sender<Vec<u8>> {
        return self.phy.0.clone();
    }

    fn insert(&mut self, channel: Sender<Vec<u8>>) -> u8 {
        let cnt = self.cnt;
        self.link.insert(self.cnt, channel);
        self.cnt += 1;
        return cnt;
    }

    fn run(&mut self) {
        if let Ok(data) = self.phy.1.recv() {
            println!("{:?} phy recv packet", data);
            let id = (data[0] + 1) % 2; // TODO: more
            if let Some(tx) = self.link.get_mut(&id) {
                let mut packet = vec![1];
                packet.extend(data[1..].to_vec());
                tx.send(packet).unwrap();
            }
        }
    }
}

fn create_new_hci(phy: &mut SimPhy, bd_addr: BDAddr) -> Sender<BTCmd> {
    let (app_tx, app_rx) = mpsc::channel();
    let (host_tx, host_rx) = mpsc::channel();
    let (bb_tx, bb_rx) = mpsc::channel();

    let id = phy.insert(bb_tx.clone());

    let mut bb = baseband::Control::new(id);
    bb.set_upper_sender(host_tx);
    bb.set_lower_sender(phy.get_phy().clone());

    bb.set_upper_send_packet(|this, data| if let Some(tx) = this.get_upper_sender() {});
    bb.set_lower_send_packet(|this, data| {
        if let Some(tx) = this.get_lower_sender() {
            let mut packet = vec![this.id];
            packet.extend(data);
            tx.send(packet).unwrap();
        }
    });

    // baseband
    thread::spawn(move || loop {
        let packet = bb_rx.recv().unwrap();
        println!("{:?} bb {} recv packet", packet, bb.id);

        let from = packet[0];
        let packet = packet[1..].to_vec();
        if from == 0 {
            bb.recv_host_packet(packet);
        } else {
            bb.recv_phy_packet(packet)
        }
    });

    let mut hci: Hci<Sender<Vec<u8>>> = Hci::new(bd_addr);
    hci.set_sender(bb_tx);
    hci.set_send_packet(|this, packet, data| {
        if let Some(tx) = this.get_sender() {
            println!("{:?} host send packet", this.get_bd_addr());
            let mut tmp = vec![0, packet as u8];
            tmp.extend(this.get_bd_addr());
            tmp.extend(data);
            tx.send(tmp).unwrap();
        }
    });

    // host
    thread::spawn(move || {
        let mut cmd: Option<BTCmd> = None;
        loop {
            if cmd.is_none() {
                cmd = Some(app_rx.recv().unwrap());
            }
            println!("{:?} recv app", hci.get_bd_addr());
            cmd.unwrap().exec(&mut hci);

            println!("{:?} loop start", hci.get_bd_addr());
            loop {
                let data = match host_rx.try_recv() {
                    Ok(data) => Some(data),
                    Err(_) => None,
                };

                // process data
                if let Some(data) = data {
                    println!("{:?} recv data", hci.get_bd_addr());
                    if data.len() > 0 {
                        let packet = data[0];
                        let data = (&data[1..]).to_owned();
                        match packet {
                            x if x == HciPacket::Command as u8 => hci.recv_ce_data(data),
                            x if x == HciPacket::ACL as u8 => hci.recv_acl_data(data),
                            _ => panic!("error packet"),
                        }
                    }
                }

                // do something here

                // check if there is a command
                cmd = app_rx.try_recv().map_or(None, |cmd| Some(cmd));
                if cmd.is_some() {
                    break;
                }
            }
            println!("{:?} loop end", hci.get_bd_addr());
        }
    });
    return app_tx;
}

fn main() {
    env_logger::init();
    log::debug!("debug log test");

    let mut sim_bb = SimPhy::new();

    let addr1 = [1, 0, 0, 0, 0, 0];
    let addr2 = [2, 0, 0, 0, 0, 0];
    let app1 = create_new_hci(&mut sim_bb, addr1);
    let app2 = create_new_hci(&mut sim_bb, addr2);

    // just test
    app1.send(BTCmd::On).unwrap();
    app2.send(BTCmd::On).unwrap();

    let bb: thread::JoinHandle<_> = thread::spawn(move || loop {
        sim_bb.run();
    });

    app1.send(BTCmd::Connect(addr2)).unwrap();

    // pend
    bb.join().unwrap();
}
