use crate::BDAddr;
use alloc::vec::Vec;
use log::info;

pub struct Hci<T> {
    bd_addr: BDAddr,
    sender: Option<T>,
    send_packet: Option<fn(&Self, u8, Vec<u8>)>,
}

impl<T> Hci<T> {
    pub fn new(bd_addr: BDAddr) -> Self {
        Hci {
            bd_addr,
            sender: None,
            send_packet: None,
        }
    }

    pub fn get_bd_addr(&self) -> BDAddr {
        return self.bd_addr;
    }

    pub fn set_sender(&mut self, sender: T) {
        self.sender = Some(sender);
    }

    pub fn get_sender(&self) -> &Option<T> {
        return &self.sender;
    }

    pub fn set_send_packet(&mut self, send_packet: fn(&Self, u8, Vec<u8>)) {
        self.send_packet = Some(send_packet);
    }
    pub fn recv_ce_data(&mut self, data: Vec<u8>) {
        info!("CE {:?}", data);
    }
    pub fn recv_acl_data(&mut self, data: Vec<u8>) {
        info!("ACL {:?}", data);
    }

    // fn send_ce_data(&mut self, data: Vec<u8>) {
    //     if let Some(send) = self.send_packet {
    //         send(&self, 1, data);
    //     }
    // }

    // fn send_acl_data(&mut self, data: Vec<u8>) {
    //     if let Some(send) = self.send_packet {
    //         send(&self, 2, data);
    //     }
    // }
}
