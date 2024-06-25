pub mod ble;
pub mod bt;

use alloc::vec::Vec;

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
        if let Some(send) = self.lower_send_packet {
            send(&self, packet);
        }
    }
}
