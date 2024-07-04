pub mod ble;
pub mod bt;
mod hci;

use alloc::vec;
use alloc::vec::Vec;
use log::info;

use hci::HCI_CMD_TABLE;

use crate::host::hci::{opcode_to_ocf, opcode_to_ogf, HCIPacket};

pub struct Control {
    pub id: u8,
    upper_send_packet: Option<fn(&Self, Vec<u8>)>,
    lower_send_packet: Option<fn(&Self, Vec<u8>)>,
}

impl Control {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            upper_send_packet: None,
            lower_send_packet: None,
        }
    }

    pub fn set_upper_send_packet(&mut self, send_packet: fn(&Self, Vec<u8>)) {
        self.upper_send_packet = Some(send_packet);
    }

    pub fn set_lower_send_packet(&mut self, send_packet: fn(&Self, Vec<u8>)) {
        self.lower_send_packet = Some(send_packet);
    }

    pub fn recv_phy_packet(&mut self, packet: Vec<u8>) {
        if let Some(send) = self.upper_send_packet {
            send(&self, packet);
        }
    }

    pub fn recv_host_packet(&mut self, packet: Vec<u8>) {
        if packet.len() < 3 {
            return;
        }
        let _packet_type = packet[0];
        let opcode = u16::from_le_bytes(packet[1..3].try_into().unwrap());
        let ogf = opcode_to_ogf(opcode) - 1;
        let ocf = opcode_to_ocf(opcode) - 1;
        info!("bb {} {}", ogf + 1, ocf + 1);

        if let Some(cmd) = &HCI_CMD_TABLE[ogf as usize][ocf as usize] {
            (cmd.handle)(self, opcode);
        }
    }

    fn power_on(&mut self) {}

    fn send_event(&mut self, code: u8, packet: Vec<u8>) {
        if let Some(send) = self.upper_send_packet {
            info!("bb send: {} {:?}", code, packet);
            let mut tmp = vec![HCIPacket::Event as u8, code, packet.len() as u8];
            tmp.extend(packet);
            send(&self, tmp);
        }
    }

    // fn send_to_lower(&mut self, packet: Vec<u8>) {
    //     if let Some(send) = self.lower_send_packet {
    //         send(&self, packet);
    //     }
    // }
}

#[derive(serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum ControllerErrorCode {
    Ok,
}
