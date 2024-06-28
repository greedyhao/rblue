pub mod ble;
pub mod bt;

use alloc::vec;
use alloc::vec::Vec;
use log::info;

use crate::host::hci::{
    opcode_to_ocf, opcode_to_ogf, CommandCompleteArg, ControllerAndBaseband, HCICmd, HCIEvent,
    HCIPacket,
};

pub struct Control<T> {
    pub id: u8,
    upper_sender: Option<T>,
    upper_send_packet: Option<fn(&Self, Vec<u8>)>,
    lower_sender: Option<T>,
    lower_send_packet: Option<fn(&Self, Vec<u8>)>,
}

impl<T> Control<T> {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            upper_sender: None,
            upper_send_packet: None,
            lower_sender: None,
            lower_send_packet: None,
        }
    }

    pub fn set_upper_sender(&mut self, sender: T) {
        self.upper_sender = Some(sender);
    }

    pub fn get_upper_sender(&self) -> &Option<T> {
        return &self.upper_sender;
    }

    pub fn set_upper_send_packet(&mut self, send_packet: fn(&Self, Vec<u8>)) {
        self.upper_send_packet = Some(send_packet);
    }

    pub fn set_lower_sender(&mut self, sender: T) {
        self.lower_sender = Some(sender);
    }

    pub fn get_lower_sender(&self) -> &Option<T> {
        return &self.lower_sender;
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
        let ogf = opcode_to_ogf(opcode);
        let ocf = opcode_to_ocf(opcode);
        info!("bb {} {}", ogf, ocf);

        if ogf == HCICmd::LEController as u8 {
            // ble
        } else if ogf == HCICmd::ControllerAndBaseband as u8 {
            if ocf == ControllerAndBaseband::Reset as u16 {
                info!("bb reset");
                // reset
                self.power_on();

                let evt = CommandCompleteArg {
                    num_hci_command_packets: 5,
                    opcode,
                    return_param: ControllerErrorCode::Ok,
                };

                self.send_to_upper(
                    HCIEvent::CommandComplete as u8,
                    bincode::serialize(&evt).unwrap(),
                );
            }
        }
    }

    fn power_on(&mut self) {}

    fn send_to_upper(&mut self, code: u8, packet: Vec<u8>) {
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
