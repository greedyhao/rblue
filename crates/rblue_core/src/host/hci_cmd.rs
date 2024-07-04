use super::*;

use pub_fields::pub_fields;

#[pub_fields]
#[derive(serde::Serialize)]
pub struct CommandCompleteArg<T> {
    num_hci_command_packets: u8,
    opcode: u16,
    return_param: T,
}

#[pub_fields]
#[derive(serde::Serialize)]
pub struct CreateConnectionArg {
    bd_addr: BDAddr,
    packet_type: PacketType,
    page_scan_repetition_mode: PageScanRepetitionMode,
    reserved: u8,
    clock_offset: u16,
    allow_role_switch: u8,
}

impl HCICmdSend for CreateConnectionArg {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LinkControl as u8,
            LinkControl::CreateConnection as u16,
            bincode::serialize(self).unwrap(),
        )
    }
}

pub struct ResetArg {}

impl HCICmdSend for ResetArg {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::ControllerAndBaseband as u8,
            ControllerAndBaseband::Reset as u16,
        );
    }
}

pub struct ReadLocalSupportedCommandsArg {}

impl HCICmdSend for ReadLocalSupportedCommandsArg {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::InformationalParam as u8,
            InformationalParam::ReadLocalSupportedCommands as u16,
        );
    }
}

#[pub_fields]
#[derive(serde::Serialize)]
pub struct LECreateConnectionArg {
    le_scan_interval: u16,
    le_scan_window: u16,
    initiator_filter_policy: bool,
    peer_address_type: LEAddressType,
    peer_address: BDAddr,
    own_address_type: LEAddressType,
    conn_interval_min: u16,
    conn_interval_max: u16,
    max_latency: u16,
    supervision_timeout: u16,
    min_ce_length: u16,
    max_ce_length: u16,
}

impl HCICmdSend for LECreateConnectionArg {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LECreateConnection as u16,
            bincode::serialize(self).unwrap(),
        );
    }
}
