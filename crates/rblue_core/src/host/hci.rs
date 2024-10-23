use super::hci_cmd::*;
use super::*;

use crate::alloc::borrow::ToOwned;

use alloc::collections::LinkedList;
use alloc::vec::Vec;
use log::info;

use num::ToPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};

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

#[derive(FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum ControllerAndBaseband {
    SetEventMask = 0x0001,
    Reset = 0x0003,
}

impl HCICmdOpcode for ControllerAndBaseband {
    fn get_opcode(&self) -> u16 {
        let ogf = HCICmd::ControllerAndBaseband as u8;
        into_opcode(ogf, self.to_u16().unwrap())
    }
}

#[derive(FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum InformationalParam {
    ReadLocalSupportedCommands = 0x0002,
    ReadLocalSupportedFeatures,
    ReadLocalExtendedSupportedFeatures,
    ReadBufferSize,
    ReadBDAddr = 0x0009,
}

impl HCICmdOpcode for InformationalParam {
    fn get_opcode(&self) -> u16 {
        let ogf = HCICmd::InformationalParam as u8;
        into_opcode(ogf, self.to_u16().unwrap())
    }
}

#[derive(FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum LEController {
    LESetEventMask = 0x0001,
    LEReadBufferSize = 0x0002,
    LEReadLocalSupportedFeatures,
    LESetRandomAddress = 0x0005,
    LESetAdvertisingParameters,
    LEReadAdvertisingPhysicalChannelTxPower,
    LESetAdvertisingData,
    LESetScanResponseData,
    LESetAdvertisingEnable,
    LECreateConnection = 0x000D,
}

impl HCICmdOpcode for LEController {
    fn get_opcode(&self) -> u16 {
        let ogf = HCICmd::LEController as u8;
        into_opcode(ogf, self.to_u16().unwrap())
    }
}

struct HCIConnection {
    remote: BDAddr,
    #[allow(dead_code)]
    addr_type: BDAddrType,
}

#[repr(u8)]
pub enum HCIPacket {
    Command,
    ACL,
    SCO,
    Event,
}

#[derive(PartialEq, PartialOrd)]
enum HCIState {
    Off,
    Initializing,
    Working,
    // Halting,
    // Sleeping,
    // FallingSleep,
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
    // config: HCIConfigParam,
    state: HCIState,
    sub_state: HCISubState,

    send_packet: Option<fn(&Self, HCIPacket, u16, Option<Vec<u8>>)>,

    connections: LinkedList<HCIConnection>,

    bd_addr: BDAddr,

    scan_enable: ScanEnable,

    le_advertisements_interval_min: u16,
    le_advertisements_interval_max: u16,
    le_advertisements_type: AdvertisingType,
    le_own_address_type: LEAddressType,
    le_advertisements_peer_address_type: LEAddressType,
    le_advertisements_peer_address: BDAddr,
    le_advertisements_channel_map: u8,
    le_advertisements_filter_policy: AdvertisingFilterPolicy,

    le_advertisements_state: LEAdvertisementsState,
    le_advertisements_todo: LEAdvertisementsTodo,
}

impl HCI {
    pub fn new(bd_addr: BDAddr) -> Self {
        HCI {
            // config: HCIConfigParam::default(),
            state: HCIState::Off,
            sub_state: HCISubState::SendReset,

            send_packet: None,

            connections: LinkedList::new(),

            bd_addr,

            scan_enable: ScanEnable::NoScansEnable,

            le_advertisements_interval_min: 0x0800,
            le_advertisements_interval_max: 0x0800,
            le_advertisements_type: AdvertisingType::ConnectableAndScannnable,
            le_own_address_type: LEAddressType::PublicDevice,
            le_advertisements_peer_address_type: LEAddressType::PublicDevice,
            le_advertisements_peer_address: BDAddr::default(),
            le_advertisements_channel_map: 0x07,
            le_advertisements_filter_policy: AdvertisingFilterPolicy::UnFilter,

            le_advertisements_state: LEAdvertisementsState::Idle,
            le_advertisements_todo: LEAdvertisementsTodo::Idle,
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
            return;
        }

        self.run_gap_le();
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
                let arg = ReadLocalSupportedFeaturesCmd {};
                arg.send(self);
            }
            SendSetEventMask => {
                self.sub_state = W4SendSetEventMask;
                let arg = SetEventMaskCmd { event_mask: 0 };
                arg.send(self);
            }
            SendLESetEventMask => {
                self.sub_state = W4SendLESetEventMask;
                let arg = LESetEventMaskCmd { le_event_mask: 0 };
                arg.send(self);
            }
            SendLEReadBufferSize => {
                self.sub_state = W4SendLEReadBufferSize;
                let arg = LEReadBufferSizeCmd {};
                arg.send(self);
            }
            SendReadBufferSize => {
                self.sub_state = W4SendReadBufferSize;
                let arg = ReadBufferSizeCmd {};
                arg.send(self);
            }
            SendLEReadLocalSupportedFeatures => {
                self.sub_state = W4SendLEReadLocalSupportedFeatures;
                let arg = LEReadLocalSupportedFeaturesCmd {};
                arg.send(self);
            }
            SendReadBDAddr => {
                self.sub_state = W4SendReadBDAddr;
                let arg = ReadBDAddrCmd {};
                arg.send(self);
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
                if opcode == InformationalParam::ReadLocalSupportedFeatures.get_opcode() {
                    self.sub_state = SendSetEventMask;
                }
            }
            W4SendSetEventMask => {
                if opcode == ControllerAndBaseband::SetEventMask.get_opcode() {
                    self.sub_state = SendLESetEventMask;
                }
            }
            W4SendLESetEventMask => {
                if opcode == LEController::LESetEventMask.get_opcode() {
                    self.sub_state = SendLEReadBufferSize;
                }
            }
            W4SendLEReadBufferSize => {
                if opcode == LEController::LEReadBufferSize.get_opcode() {
                    self.sub_state = SendReadBufferSize;
                }
            }
            W4SendReadBufferSize => {
                if opcode == InformationalParam::ReadBufferSize.get_opcode() {
                    self.sub_state = SendLEReadLocalSupportedFeatures;
                }
            }
            W4SendLEReadLocalSupportedFeatures => {
                if opcode == LEController::LEReadLocalSupportedFeatures.get_opcode() {
                    self.sub_state = SendReadBDAddr;
                }
            }
            W4SendReadBDAddr => {
                if opcode == InformationalParam::ReadBDAddr.get_opcode() {
                    self.sub_state = End;
                }
            }
            _ => {}
        }
    }

    fn run_gap_le(&mut self) {
        // Phase 1: collect what to stop
        let mut advertising_stop = false;
        if self
            .le_advertisements_state
            .contains(LEAdvertisementsState::Active)
        {
            if self
                .le_advertisements_todo
                .contains(LEAdvertisementsTodo::SetParams)
            {
                advertising_stop = true;
            }
        }

        // Phase 2: stop everything that should be off during modifications
        if advertising_stop {
            self.le_advertisements_state
                .remove(LEAdvertisementsState::Active);
            let cmd = LESetAdvertisingEnableCmd {
                advertiseing_enable: false,
            };
            cmd.send(self);
        }

        // Phase 3: modify
        if self
            .le_advertisements_todo
            .contains(LEAdvertisementsTodo::SetParams)
        {
            self.le_advertisements_todo
                .remove(LEAdvertisementsTodo::SetParams);
            let cmd = LESetAdvertisingParametersCmd {
                advertising_interval_min: self.le_advertisements_interval_min,
                advertising_interval_max: self.le_advertisements_interval_max,
                advertising_type: self.le_advertisements_type.clone(),
                own_address_type: self.le_own_address_type.clone(),
                peer_address_type: LEAddressType2::from(
                    self.le_advertisements_peer_address_type.clone(),
                ),
                peer_address: self.le_advertisements_peer_address,
                advertising_channel_map: self.le_advertisements_channel_map,
                advertising_filter_policy: self.le_advertisements_filter_policy.clone(),
            };
            cmd.send(self);
        }

        // Phase 4: restore state
        if self
            .le_advertisements_state
            .contains(LEAdvertisementsState::Enabled)
            && !self
                .le_advertisements_state
                .contains(LEAdvertisementsState::Active)
        {
            self.le_advertisements_state |= LEAdvertisementsState::Active;
            let cmd = LESetAdvertisingEnableCmd {
                advertiseing_enable: true,
            };
            cmd.send(self);
        }
    }

    pub fn power_control(&mut self, control: HCIPowerMode) {
        match self.state {
            HCIState::Off => self.power_control_off(control),
            _ => {}
        }
        self.run();
    }

    fn power_control_off(&mut self, control: HCIPowerMode) {
        match control {
            HCIPowerMode::On => {
                self.power_enter_initializing_state();
            }
            _ => {}
        }
    }

    fn power_enter_initializing_state(&mut self) {
        self.state = HCIState::Initializing;
        self.sub_state = HCISubState::SendReset;
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

                if self.state < HCIState::Working {
                    self.init_process_event(opcode);
                }
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

// gap

// le

pub fn gap_advertisements_set_params(
    hci: &mut HCI,
    adv_int_min: u16,
    adv_int_max: u16,
    adv_type: AdvertisingType,
    peer_addr_type: LEAddressType,
    peer_addr: BDAddr,
    channel_map: u8,
    filter_policy: AdvertisingFilterPolicy,
) {
    hci.le_advertisements_interval_min = adv_int_min;
    hci.le_advertisements_interval_max = adv_int_max;
    hci.le_advertisements_type = adv_type;
    hci.le_advertisements_peer_address_type = peer_addr_type;
    hci.le_advertisements_peer_address = peer_addr;
    hci.le_advertisements_channel_map = channel_map;
    hci.le_advertisements_filter_policy = filter_policy;
    hci.run();
}

pub fn gap_advertisements_enable(hci: &mut HCI, enable: bool) {
    if enable {
        hci.le_advertisements_state
            .insert(LEAdvertisementsState::Enabled);
    } else {
        hci.le_advertisements_state
            .remove(LEAdvertisementsState::Enabled);
    }
    hci.run();
}

// api

#[derive(Debug)]
pub enum BTCmd {
    On,
    Off,
    Connect(BDAddr),

    LEAdvtise(bool),
    LEConnect(BDAddr),
}

impl BTCmd {
    pub fn exec(&self, hci: &mut HCI) {
        info!("exec {:?}", self);
        match self {
            BTCmd::Connect(addr) => {
                for conn in hci.connections.iter() {
                    // already connected
                    if conn.remote == *addr {
                        return;
                    }
                }
                let conn = HCIConnection {
                    remote: *addr,
                    addr_type: BDAddrType::Classic,
                };
                hci.connections.push_back(conn);

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
            BTCmd::LEAdvtise(enable) => {
                if *enable {
                    hci.le_advertisements_state
                        .insert(LEAdvertisementsState::Enabled);
                } else {
                    hci.le_advertisements_state
                        .remove(LEAdvertisementsState::Enabled);
                }
                hci.run();
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
