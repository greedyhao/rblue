use crate::BDAddr;
use alloc::collections::LinkedList;
use alloc::vec::Vec;
use bitflags::bitflags;
use log::info;

#[repr(u8)]
enum LinkControlCmd {
    Inquiry = 0x01,
    InquiryCancel,
    PeriodicInquiryMod,
    ExitPeriodicInquiryMod,
    CreateConnection(CreateConnectionArg),
    Disconnect,
    AcceptConnectionRequest,
    RejectConnectionRequest,
}

impl LinkControlCmd {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

enum HciCmd {
    LinkControl(LinkControlCmd),
}

impl HciCmd {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

#[derive(serde::Serialize)]
struct CreateConnectionArg {
    bd_addr: BDAddr,
    packet_type: PacketType,
    page_scan_repetition_mode: PageScanRepetitionMode,
    reserved: u8,
    clock_offset: u16,
    allow_role_switch: u8,
}

bitflags! {
    struct PacketType: u16 {
        const NoUse2DH1 = 0x0001;
        const NoUse3DH1 = 0x0002;
        const MayUseDM1 = 0x0004;
        const MayUseDH1 = 0x0008;
        const NoUse2DH3 = 0x0100;
        const NoUse3DH3 = 0x0200;
        const MayUseDM3 = 0x0400;
        const MayUseDH3 = 0x0800;
        const NoUse2DH5 = 0x1000;
        const NoUse3DH5 = 0x2000;
        const MayUseDM5 = 0x4000;
        const MayUseDH5 = 0x8000;
    }
}

impl serde::Serialize for PacketType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = serializer.serialize_u16(self.bits());
        s
    }
}

#[derive(serde_repr::Serialize_repr)]
#[repr(u8)]
enum PageScanRepetitionMode {
    R0 = 0,
    R1,
    R2,
}

#[repr(u8)]
enum ScanEnable {
    NoScansEnable,
    InquiryEnablePageDisable,
    InquiryDisablePageEnable,
    InquiryEnablePageEnable,
}

struct HciConnection {
    remote: BDAddr,
}

#[repr(u8)]
pub enum HciPacket {
    Command,
    ACL,
    SCO,
    Event,
}

pub struct Hci<T> {
    sender: Option<T>,
    send_packet: Option<fn(&Self, HciPacket, Vec<u8>)>,

    connections: LinkedList<HciConnection>,

    bd_addr: BDAddr,

    scan_enable: ScanEnable,
}

impl<T> Hci<T> {
    pub fn new(bd_addr: BDAddr) -> Self {
        Hci {
            sender: None,
            send_packet: None,

            connections: LinkedList::new(),

            bd_addr,

            scan_enable: ScanEnable::NoScansEnable,
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

    pub fn set_send_packet(&mut self, send_packet: fn(&Self, HciPacket, Vec<u8>)) {
        self.send_packet = Some(send_packet);
    }

    pub fn recv_ce_data(&mut self, data: Vec<u8>) {
        info!("CE {:?}", data);
    }
    pub fn recv_acl_data(&mut self, data: Vec<u8>) {
        info!("ACL {:?}", data);
    }

    fn send_cmd(&mut self, cmd: HciCmd) {
        let ogf = cmd.discriminant();
        match cmd {
            HciCmd::LinkControl(sub) => {
                let ocf = sub.discriminant();
                match sub {
                    LinkControlCmd::CreateConnection(arg) => {
                        let mut data = Vec::new();
                        data.extend(into_opcode(ogf, ocf).to_le_bytes());
                        let tmp = bincode::serialize(&arg).unwrap();
                        data.extend(tmp);
                        self.send_cmd_data(data);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn send_cmd_data(&mut self, data: Vec<u8>) {
        info!("send cmd {:?}", data);
        if let Some(send) = self.send_packet {
            send(&self, HciPacket::Command, data);
        }
    }

    // fn send_acl_data(&mut self, data: Vec<u8>) {
    //     if let Some(send) = self.send_packet {
    //         send(&self, 2, data);
    //     }
    // }
}

#[derive(Debug)]
pub enum BTCmd {
    On,
    Off,
    Connect(BDAddr),
}

impl BTCmd {
    pub fn exec<T>(&self, hci: &mut Hci<T>) {
        info!("exec {:?}", self);
        match self {
            BTCmd::On => {
                hci.scan_enable = ScanEnable::InquiryEnablePageEnable;
            }
            BTCmd::Connect(addr) => {
                for conn in hci.connections.iter() {
                    // already connected
                    if conn.remote == *addr {
                        return;
                    }
                }

                // create connection
                let arg = CreateConnectionArg {
                    bd_addr: *addr,
                    packet_type: PacketType::MayUseDH1,
                    page_scan_repetition_mode: PageScanRepetitionMode::R0,
                    reserved: 0,
                    clock_offset: 0,
                    allow_role_switch: 1,
                };
                hci.send_cmd(HciCmd::LinkControl(LinkControlCmd::CreateConnection(arg)));
            }
            _ => {}
        }
    }
}

fn into_opcode(ogf: u8, ocf: u8) -> u16 {
    return (ogf as u16) << 10 | (ocf as u16);
}
