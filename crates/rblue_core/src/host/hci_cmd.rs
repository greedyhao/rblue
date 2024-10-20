use super::*;

use pub_fields::pub_fields;
extern crate rblue_proc_macro;
use rblue_proc_macro::ToU8Array;
use rblue_proc_macro::FromBytes;

pub trait RBlueToU8Array {
    fn to_u8_array(&self) -> Vec<u8>;
}

pub trait RBlueFromU8Array: Sized {
    fn from_u8_array(bytes: &[u8]) -> Option<Self>;
}

#[pub_fields]
pub struct CommandCompleteEvt<T> {
    num_hci_command_packets: u8,
    opcode: u16,
    return_param: T,
}

impl<T> CommandCompleteEvt<T>
where
    T: RBlueToU8Array,
{
    pub fn to_u8_array(&self) -> Vec<u8> {
        let mut array = alloc::vec![self.num_hci_command_packets];
        array.extend_from_slice(&self.opcode.to_le_bytes());
        array.extend(self.return_param.to_u8_array());
        array
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct CreateConnectionCmd {
    bd_addr: BDAddr,
    packet_type: PacketType,
    page_scan_repetition_mode: PageScanRepetitionMode,
    reserved: u8,
    clock_offset: u16,
    allow_role_switch: u8,
}

impl HCICmdSend for CreateConnectionCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LinkControl as u8,
            LinkControl::CreateConnection as u16,
            self.to_u8_array(),
        )
    }
}

// Controller and Baseband Commands

#[derive(ToU8Array)]
#[pub_fields]
pub struct SetEventMaskCmd {
    event_mask: u64,
}

impl HCICmdSend for SetEventMaskCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::ControllerAndBaseband as u8,
            ControllerAndBaseband::SetEventMask as u16,
        );
    }
}

#[derive(ToU8Array)]
#[pub_fields]
pub struct SetEventMaskRet {
    status: ControllerErrorCode,
}

pub struct ResetCmd {}

impl HCICmdSend for ResetCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::ControllerAndBaseband as u8,
            ControllerAndBaseband::Reset as u16,
        );
    }
}

#[derive(ToU8Array)]
#[pub_fields]
pub struct ResetRet {
    status: ControllerErrorCode,
}

// Informational Parameters

pub struct ReadLocalSupportedCommandsCmd {}

impl HCICmdSend for ReadLocalSupportedCommandsCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::InformationalParam as u8,
            InformationalParam::ReadLocalSupportedCommands as u16,
        );
    }
}

#[derive(ToU8Array)]
#[pub_fields]
pub struct ReadLocalSupportedCommandsRet {
    status: ControllerErrorCode,
    supported_commands: SupportedCommands,
}

#[derive(ToU8Array)]
pub struct ReadLocalSupportedFeaturesCmd {}

impl HCICmdSend for ReadLocalSupportedFeaturesCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::InformationalParam as u8,
            InformationalParam::ReadLocalSupportedFeatures as u16,
        );
    }
}

#[derive(ToU8Array)]
#[pub_fields]
pub struct ReadLocalSupportedFeaturesRet {
    status: ControllerErrorCode,
    lmp_feature: LMPFeatures,
}

#[derive(ToU8Array)]
pub struct ReadBufferSizeCmd {}

impl HCICmdSend for ReadBufferSizeCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::InformationalParam as u8,
            InformationalParam::ReadBufferSize as u16,
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct ReadBufferSizeRet {
    status: ControllerErrorCode,
    acl_data_packet_length: u16,
    synchronous_data_packet_length: u8,
    total_num_acl_data_packets: u16,
    total_num_synchronous_data_packets: u16,
}

#[derive(ToU8Array)]
pub struct ReadBDAddrCmd {}

impl HCICmdSend for ReadBDAddrCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::InformationalParam as u8,
            InformationalParam::ReadBDAddr as u16,
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct ReadBDAddrRet {
    status: ControllerErrorCode,
    bd_addr: BDAddr,
}

// LE Controller Commands

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetEventMaskCmd {
    le_event_mask: u64,
}

impl HCICmdSend for LESetEventMaskCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LESetEventMask as u16,
            self.to_u8_array(),
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetEventMaskRet {
    status: ControllerErrorCode,
}

#[derive(ToU8Array)]
pub struct LEReadBufferSizeCmd {}

impl HCICmdSend for LEReadBufferSizeCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::LEController as u8,
            LEController::LEReadBufferSize as u16,
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LEReadBufferSizeRet {
    status: ControllerErrorCode,
    le_acl_data_packet_length: u16,
    total_num_le_acl_data_packets: u8,
}

#[derive(ToU8Array)]
pub struct LEReadLocalSupportedFeaturesCmd {}

impl HCICmdSend for LEReadLocalSupportedFeaturesCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::LEController as u8,
            LEController::LEReadLocalSupportedFeatures as u16,
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LEReadLocalSupportedFeaturesRet {
    status: ControllerErrorCode,
    le_features: u64,
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetAdvertisingParametersCmd {
    advertising_interval_min: u16, // 0x0020 - 0x4000
    advertising_interval_max: u16, // 0x0020 - 0x4000
    advertising_type: AdvertisingType,
    own_address_type: LEAddressType,
    peer_address_type: LEAddressType2,
    peer_address: BDAddr,
    advertising_channel_map: u8,
    advertising_filter_policy: AdvertisingFilterPolicy,
}

impl HCICmdSend for LESetAdvertisingParametersCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LESetAdvertisingParameters as u16,
            self.to_u8_array(),
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetAdvertisingParametersRet {
    status: ControllerErrorCode,
}

#[derive(ToU8Array)]
pub struct LEReadAdvertisingPhysicalChannelTxPowerCmd {}

impl HCICmdSend for LEReadAdvertisingPhysicalChannelTxPowerCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_no_param(
            HCICmd::LEController as u8,
            LEController::LEReadAdvertisingPhysicalChannelTxPower as u16,
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LEReadAdvertisingPhysicalChannelTxPowerRet {
    status: ControllerErrorCode,
    tx_power_level: u8,
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetAdvertisingDataCmd {
    advertising_data_length: u8,
    advertising_data: LEAdvPacket,
}

impl HCICmdSend for LESetAdvertisingDataCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LESetAdvertisingData as u16,
            self.to_u8_array(),
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetAdvertisingDataRet {
    status: ControllerErrorCode,
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetScanResponseDataCmd {
    scan_response_data_length: u8,
    scan_response_data: LEAdvPacket,
}

impl HCICmdSend for LESetScanResponseDataCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LESetScanResponseData as u16,
            self.to_u8_array(),
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetScanResponseDataRet {
    status: ControllerErrorCode,
}

#[pub_fields]
#[derive(ToU8Array, FromBytes)]
pub struct LESetAdvertisingEnableCmd {
    advertiseing_enable: bool,
}

impl HCICmdSend for LESetAdvertisingEnableCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LESetAdvertisingEnable as u16,
            self.to_u8_array(),
        );
    }
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LESetAdvertisingEnableRet {
    status: ControllerErrorCode,
}

#[pub_fields]
#[derive(ToU8Array)]
pub struct LECreateConnectionCmd {
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

impl HCICmdSend for LECreateConnectionCmd {
    fn send(&self, hci: &mut HCI) {
        hci.send_cmd_with_param(
            HCICmd::LEController as u8,
            LEController::LECreateConnection as u16,
            self.to_u8_array(),
        );
    }
}
