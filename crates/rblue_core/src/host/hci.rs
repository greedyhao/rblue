use super::hci_cmd::*;
use super::*;

use crate::alloc::borrow::ToOwned;

use alloc::collections::LinkedList;
use alloc::vec::Vec;
use log::info;

use num_derive::FromPrimitive;

trait HCICmdOpcode {
    fn get_opcode(&self) -> u16;
}

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum HCICmd {
    LinkControl = 0x0001,
    LinkPolicy,
    ControllerAndBaseband,
    InformationalParam,
    StatusParam,
    TestingCommand,
    LEController = 0x0008,
}

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum HCIEvent {
    CommandComplete = 0x0E,
}

#[repr(u16)]
pub enum LinkControl {
    Inquiry = 0x0001,
    InquiryCancel,
    PeriodicInquiryMod,
    ExitPeriodicInquiryMod,
    CreateConnection,
    Disconnect,
    AcceptConnectionRequest,
    RejectConnectionRequest,
}

impl HCICmdOpcode for LinkControl {
    fn get_opcode(&self) -> u16 {
        match self {
            _ => {}
        }
        0
    }
}

#[derive(Clone, Copy, FromPrimitive)]
#[repr(u16)]
pub enum ControllerAndBaseband {
    SetEventMask = 0x0001,
    Reset = 0x0003,
}

impl HCICmdOpcode for ControllerAndBaseband {
    fn get_opcode(&self) -> u16 {
        let ogf = HCICmd::ControllerAndBaseband as u8;
        into_opcode(ogf, *self as u16)
    }
}

#[derive(Clone, Copy, FromPrimitive)]
#[repr(u16)]
pub enum InformationalParam {
    ReadLocalSupportedCommands = 0x0002,
}

impl HCICmdOpcode for InformationalParam {
    fn get_opcode(&self) -> u16 {
        let ogf = HCICmd::InformationalParam as u8;
        into_opcode(ogf, *self as u16)
    }
}

#[repr(u16)]
pub enum LEController {
    LECreateConnection = 0x000D,
}


struct HCIConnection {
    remote: BDAddr,
}

#[repr(u8)]
pub enum HCIPacket {
    Command,
    ACL,
    SCO,
    Event,
}

#[derive(PartialEq)]
enum HCIState {
    Off,
    Initializing,
    Working,
}

#[derive(PartialEq)]
enum HCISubState {
    SendReset,
    W4SendReset,
    SendReadLocalSupportedCommands,
    W4SendReadLocalSupportedCommands,
    SendReadLocalSupportedFeatures,
    W4SendReadLocalSupportedFeatures,
    SendSetEventMask,
    W4SendSetEventMask,
    SendLESetEventMask,
    W4SendLESetEventMask,
    SendLEReadBufferSize,
    W4SendLEReadBufferSize,
    SendReadBufferSize,
    W4SendReadBufferSize,
    SendLEReadLocalSupportedFeatures,
    W4SendLEReadLocalSupportedFeatures,
    SendReadBDAddr,
    W4SendReadBDAddr,
    End,
}

pub struct HCI {
    state: HCIState,
    sub_state: HCISubState,

    send_packet: Option<fn(&Self, HCIPacket, u16, Option<Vec<u8>>)>,

    connections: LinkedList<HCIConnection>,

    bd_addr: BDAddr,

    scan_enable: ScanEnable,
}

impl HCI {
    pub fn new(bd_addr: BDAddr) -> Self {
        HCI {
            state: HCIState::Off,
            sub_state: HCISubState::SendReset,

            send_packet: None,

            connections: LinkedList::new(),

            bd_addr,

            scan_enable: ScanEnable::NoScansEnable,
        }
    }

    pub fn get_bd_addr(&self) -> BDAddr {
        return self.bd_addr;
    }

    pub fn set_send_packet(&mut self, send_packet: fn(&Self, HCIPacket, u16, Option<Vec<u8>>)) {
        self.send_packet = Some(send_packet);
    }

    pub fn recv_packet(&mut self, packet: Vec<u8>) {
        let data = packet[1..].to_owned();
        match packet[0] {
            x if x == HCIPacket::Command as u8 => self.recv_ce_data(data),
            x if x == HCIPacket::ACL as u8 => self.recv_acl_data(data),
            x if x == HCIPacket::Event as u8 => self.recv_event_data(data),
            _ => panic!("error packet"),
        }
        self.run();
    }

    fn run(&mut self) {
        if self.state == HCIState::Initializing {
            self.init_process();
        }
    }

    fn init_process(&mut self) {
        use HCISubState::*;
        match self.sub_state {
            SendReset => {
                self.sub_state = W4SendReset;
                let arg = ResetCmd {};
                arg.send(self);
            }
            SendReadLocalSupportedCommands => {
                self.sub_state = W4SendReadLocalSupportedCommands;
                let arg = ReadLocalSupportedCommandsCmd {};
                arg.send(self);
            }
            SendReadLocalSupportedFeatures => {
                self.sub_state = W4SendReadLocalSupportedFeatures;
            }
            SendSetEventMask => {
                self.sub_state = W4SendSetEventMask;
            }
            SendLESetEventMask => {
                self.sub_state = W4SendLESetEventMask;
            }
            SendLEReadBufferSize => {
                self.sub_state = W4SendLEReadBufferSize;
            }
            SendReadBufferSize => {
                self.sub_state = W4SendReadBufferSize;
            }
            SendLEReadLocalSupportedFeatures => {
                self.sub_state = W4SendLEReadLocalSupportedFeatures;
            }
            SendReadBDAddr => {
                self.sub_state = W4SendReadBDAddr;
            }
            End => {
                self.state = HCIState::Working;
                info!("HCI init done: {:?}", self.bd_addr);
            }
            _ => {}
        }
    }

    fn init_process_event(&mut self, opcode: u16) {
        use HCISubState::*;
        match self.sub_state {
            W4SendReset => {
                if opcode == ControllerAndBaseband::Reset.get_opcode() {
                    self.sub_state = SendReadLocalSupportedCommands;
                }
            }
            W4SendReadLocalSupportedCommands => {
                if opcode == InformationalParam::ReadLocalSupportedCommands.get_opcode() {
                    self.sub_state = SendReadLocalSupportedFeatures;
                }
            }
            W4SendReadLocalSupportedFeatures => {
                self.sub_state = SendSetEventMask;
            }
            W4SendSetEventMask => {
                self.sub_state = SendLESetEventMask;
            }
            W4SendLESetEventMask => {
                self.sub_state = SendLEReadBufferSize;
            }
            W4SendLEReadBufferSize => {
                self.sub_state = SendReadBufferSize;
            }
            W4SendReadBufferSize => {
                self.sub_state = SendLEReadLocalSupportedFeatures;
            }
            W4SendLEReadLocalSupportedFeatures => {
                self.sub_state = SendReadBDAddr;
            }
            W4SendReadBDAddr => {
                self.sub_state = End;
            }
            _ => {}
        }
    }

    fn power_on(&mut self) {
        self.state = HCIState::Initializing;
        self.sub_state = HCISubState::SendReset;
        self.run();
    }

    fn recv_ce_data(&mut self, data: Vec<u8>) {
        info!("CE {:?}", data);
    }
    fn recv_acl_data(&mut self, data: Vec<u8>) {
        info!("ACL {:?}", data);
    }
    fn recv_event_data(&mut self, data: Vec<u8>) {
        info!("EV {:?}", data);
        let event = num::FromPrimitive::from_u8(data[0]);
        match event {
            Some(HCIEvent::CommandComplete) => {
                let opcode = u16::from_le_bytes(data[3..5].try_into().unwrap());
                self.init_process_event(opcode);
            }
            _ => {}
        }
    }

    pub fn send_cmd_no_param(&mut self, ogf: u8, ocf: u16) {
        info!("send cmd {} {}", ogf, ocf);
        if let Some(send) = self.send_packet {
            send(&self, HCIPacket::Command, into_opcode(ogf, ocf), None);
        }
    }
    pub fn send_cmd_with_param(&mut self, ogf: u8, ocf: u16, param: Vec<u8>) {
        info!("send cmd {} {} {:?}", ogf, ocf, param);
        if let Some(send) = self.send_packet {
            send(
                &self,
                HCIPacket::Command,
                into_opcode(ogf, ocf),
                Some(param),
            );
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

    LEConnect(BDAddr),
}

impl BTCmd {
    pub fn exec(&self, hci: &mut HCI) {
        info!("exec {:?}", self);
        match self {
            BTCmd::On => {
                hci.scan_enable = ScanEnable::InquiryEnablePageEnable;
                hci.power_on();
            }
            BTCmd::Connect(addr) => {
                for conn in hci.connections.iter() {
                    // already connected
                    if conn.remote == *addr {
                        return;
                    }
                }

                // create connection
                let arg = CreateConnectionCmd {
                    bd_addr: *addr,
                    packet_type: PacketType::MayUseDH1,
                    page_scan_repetition_mode: PageScanRepetitionMode::R0,
                    reserved: 0,
                    clock_offset: 0,
                    allow_role_switch: 1,
                };
                arg.send(hci);
            }
            BTCmd::LEConnect(addr) => {
                for conn in hci.connections.iter() {
                    // already connected
                    if conn.remote == *addr {
                        return;
                    }
                }
                let arg = LECreateConnectionCmd {
                    le_scan_interval: 16,
                    le_scan_window: 16,
                    initiator_filter_policy: false,
                    peer_address_type: LEAddressType::PublicDevice,
                    peer_address: *addr,
                    own_address_type: LEAddressType::PublicDevice,
                    conn_interval_min: 6,
                    conn_interval_max: 7,
                    max_latency: 0,
                    supervision_timeout: 10,
                    min_ce_length: 0,
                    max_ce_length: 0,
                };
                arg.send(hci);
            }
            _ => {}
        }
    }
}

fn into_opcode(ogf: u8, ocf: u16) -> u16 {
    return (ogf as u16) << 10 | ocf;
}

pub fn opcode_to_ogf(opcode: u16) -> u8 {
    return (opcode >> 10) as u8;
}

pub fn opcode_to_ocf(opcode: u16) -> u16 {
    return opcode & 0x3ff;
}
