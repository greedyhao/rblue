use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        OnceLock,
    },
    thread,
};

use rblue_core::{
    baseband::{self, Control},
    host::hci::*,
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

fn create_new_hci(
    phy: &mut SimPhy,
    bd_addr: BDAddr,
    bridge: &OnceLock<RBlueBridge>,
    cb: &RBlueBridgeCB,
) {
    let (app_tx, app_rx) = mpsc::channel();
    let (host_tx, host_rx) = mpsc::channel();
    let (bb_tx, bb_rx) = mpsc::channel();

    let sim = RBlueBridge {
        app_to_host: app_tx.clone(),
        bb_to_host: host_tx.clone(),
        host_to_bb: bb_tx.clone(),
        bb_to_phy: phy.get_phy().clone(),
    };
    bridge.set(sim).unwrap();

    let id = phy.insert(bb_tx.clone());

    let mut bb = baseband::Control::new(id);

    bb.set_upper_send_packet(cb.bb_to_host);
    bb.set_lower_send_packet(cb.bb_to_phy);

    // baseband
    thread::spawn(move || loop {
        let packet = bb_rx.recv().unwrap();

        let from = packet[0];
        let packet = packet[1..].to_vec();
        if from == 0 {
            println!("{:?} host->bb {:?}", bb.id, packet);
            bb.recv_host_packet(packet);
        } else {
            println!("{:?} phy->bb {:?}", bb.id, packet);
            bb.recv_phy_packet(packet)
        }
    });

    let mut hci = HCI::new(bd_addr);
    hci.set_send_packet(cb.host_to_bb);

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
                        hci.recv_packet(data);
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
}

#[derive(Debug)]
struct RBlueBridge {
    app_to_host: Sender<BTCmd>,
    host_to_bb: Sender<Vec<u8>>,
    bb_to_phy: Sender<Vec<u8>>,
    bb_to_host: Sender<Vec<u8>>,
}

struct RBlueBridgeCB {
    host_to_bb: fn(&HCI, HCIPacket, u16, Option<Vec<u8>>),
    bb_to_phy: fn(&Control, Vec<u8>),
    bb_to_host: fn(&Control, Vec<u8>),
}

static APP1_SIM: OnceLock<RBlueBridge> = OnceLock::new();
static APP2_SIM: OnceLock<RBlueBridge> = OnceLock::new();

macro_rules! create_sim_stack_cb {
    ($app_sim:expr) => {
        RBlueBridgeCB {
            host_to_bb: |hci, packet, opcode, param| {
                println!("{:?} host send packet", hci.get_bd_addr());
                if let Some(tx) = $app_sim.get() {
                    let mut tmp = vec![0, packet as u8];
                    tmp.extend(opcode.to_le_bytes());
                    if let Some(param) = param {
                        tmp.extend(param);
                    }
                    tx.host_to_bb.send(tmp).unwrap();
                }
            },
            bb_to_phy: |this, data| {
                if let Some(tx) = $app_sim.get() {
                    println!("{:?} bb->phy {:?}", this.id, data);
                    tx.bb_to_phy.send(data).unwrap();
                }
            },
            bb_to_host: |this, data| {
                if let Some(tx) = $app_sim.get() {
                    println!("{:?} bb->host {:?}", this.id, data);
                    tx.bb_to_host.send(data).unwrap();
                }
            },
        }
    };
}

fn main() {
    env_logger::init();
    log::debug!("debug log test");

    let mut sim_bb = SimPhy::new();

    let cb1 = create_sim_stack_cb!(APP1_SIM);
    let addr1 = [1, 0, 0, 0, 0, 0];
    create_new_hci(&mut sim_bb, addr1, &APP1_SIM, &cb1);

    let cb2 = create_sim_stack_cb!(APP2_SIM);
    let addr2 = [2, 0, 0, 0, 0, 0];
    create_new_hci(&mut sim_bb, addr2, &APP2_SIM, &cb2);

    // just test
    APP1_SIM.get().unwrap().app_to_host.send(BTCmd::On).unwrap();
    APP2_SIM.get().unwrap().app_to_host.send(BTCmd::On).unwrap();

    let bb: thread::JoinHandle<_> = thread::spawn(move || loop {
        sim_bb.run();
    });

    // pend
    bb.join().unwrap();
}
