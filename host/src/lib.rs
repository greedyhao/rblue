#![cfg_attr(not(test), no_std)]

use rblue_hci_protocol::*;

struct Host<T: HciDevice> {
    hcidevice: T,
    hci_buf: [u8; 256],
}

impl<T: HciDevice> Host<T> {
    fn new(hcidevice: T) -> Self {
        Self { hcidevice, hci_buf: [0; 256] }
    }

    fn run_once(&mut self) {
        self.hcidevice.recv_packet(&mut self.hci_buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{self, Receiver, Sender};

    struct MockHciDevice {
        tx: Sender<Vec<u8>>,
        rx: Receiver<Vec<u8>>,
        packet_handler: Option<PacketHandler>,
    }
    impl HciDevice for MockHciDevice {
        fn open(&self) -> Result<(), Error> {
            Ok(())
        }
        fn close(&self) -> Result<(), Error> {
            Ok(())
        }
        fn register_packet_handler(&mut self, handler: PacketHandler) {
            self.packet_handler = Some(handler);
        }
        fn can_send_packet_now(&self, packet_type: PacketType) -> bool {
            true
        }
        fn send_packet(&self, packet_type: PacketType, packet: &[u8]) -> Result<(), Error> {
            Ok(())
        }
        fn recv_packet(&mut self, packet: &mut [u8]) {
            if let Ok(p) = self.rx.try_recv() {
                // println!("Received packet: {:?}", p);
                packet[..p.len()].copy_from_slice(&p);
            }
        }
        fn set_baudrate(&self, baudrate: u32) -> Result<(), Error> {
            Ok(())
        }
    }
    impl MockHciDevice {
        fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> Self {
            Self {
                tx,
                rx,
                packet_handler: None,
            }
        }
    }

    #[test]
    fn host_init() {
        let h2c = mpsc::channel();
        let c2h = mpsc::channel();
        let hcidevice = MockHciDevice::new(h2c.0, c2h.1);
        let mut host = Host::new(hcidevice);

        c2h.0.send(vec![0x01, 0x02, 0x03]).unwrap();
        host.run_once();
        println!("Host received packet: {:?}", host.hci_buf);
    }
}
