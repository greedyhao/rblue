pub mod hci;
pub mod hci_cmd;

pub use crate::BDAddr;
use alloc::vec::Vec;
pub use hci::HCICmd;

pub use hci::LinkControl;
// pub use hci::LinkPolicy;
pub use hci::ControllerAndBaseband;
pub use hci::InformationalParam;
// pub use hci::StatusParam;
// pub use hci::TestingCommand;
pub use hci::LEController;

pub use crate::baseband::ControllerErrorCode;

use bitflags::bitflags;
use rblue_proc_macro::EnumU8ToLeBytes;

type SupportedCommands = [u8; 64];
type LMPFeatures = [u8; 8];
type LEAdvPacket = [u8; 31];

bitflags! {
    pub struct PacketType: u16 {
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

impl PacketType {
    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.bits().to_le_bytes()
    }
}

// pub struct HCIConfigParam {
//     scan_enable: ScanEnable,
//     /// Range: 0x0012 to 0x1000; only even values are valid, unit: 0.625ms
//     inquiry_scan_interval: u16,
//     /// Range: 0x0011 to 0x1000(Mandatory Range: 0x0011 to Inquiry Scan Interval), unit: 0.625ms
//     inquiry_scan_window: u16,
//     inquiry_scan_type: ScanType,
//     inquiry_mode: InquiryMode,
//     /// Range: 0x0001 to 0xFFFF(Mandatory Range: 0x0016 to 0xFFFF), unit: 0.625ms
//     page_timeout: u16,
//     /// Range: 0x0001 to 0xB540(Mandatory Range: 0x00A0 to 0xB540), unit: 0.625ms
//     connection_accept_timeout: u16,
//     /// Range: 0x0012 to 0x1000; only even values are valid, unit: 0.625ms
//     page_scan_interval: u16,
//     /// Range: 0x0011 to 0x1000(Mandatory Range: 0x0011 to Inquiry Scan Interval), unit: 0.625ms
//     page_scan_window: u16,
//     page_scan_type: ScanType,
//     voice_setting: u8,
//     pin_type: PinType,
//     // link_key:

//     // Todo: reset counter
//     failed_contact_counter: u16,
//     authentication_enable: AuthenticationEnable,
//     hold_mode_activity: HoldModeActivity,
//     /// bit0:Role switch bit1:Hold mode bit2:Sniff mode
//     link_policy_settings: u8,
//     /// zero as ∞, Range: 0x0001 to 0x07FF(Mandatory Range: 0x0002 to 0x07FF), unit: 0.625ms
//     flush_timeout: u16,
//     /// Range: 0x00 to 0xFE
//     num_broadcast_retransmissions: u8,
//     /// zero or range from 0x0001 to 0xFFFF(Mandatory Range: 0x0190 to 0xFFFF)
//     link_supervision_timeout: u16,
//     synchronous_flow_control_enable: bool,
//     // local_name:
//     // EXTENDED INQUIRY RESPONSE
//     errorneous_data_reporting: bool,
//     class_of_device: u32,
//     supported_commands: SupportedCommands,
//     flow_control_mode: FlowControlMode,
//     le_supported_host: bool,
//     /// Range: 0x0020 to 0xFFFE; only even values are valid(Mandatory Range: 0x0020 to 0x1000), unit: 0.625ms
//     sync_train_interval: u16,
//     /// Range: 0x00000002 to 0x07FFFFFE; only even values are valid, unit: 0.625ms
//     sync_train_timeout: u32,
//     service_data: u8,
//     secure_connections_host_support: bool,
//     /// Range: 0x0001 to 0xFFFF, unit: 0.625ms
//     authentication_payload_timeout: u16,
//     /// Range: 0x0000 to 0xFFFF, unit: 0.625ms
//     extended_page_timeout: u16,
//     /// Range: 0x0000 to 0xFFFF
//     extended_inquiry_length: u16,
// }

// impl Default for HCIConfigParam {
//     fn default() -> Self {
//         Self {
//             scan_enable: ScanEnable::NoScansEnable,
//             inquiry_scan_interval: 0x1000,
//             inquiry_scan_window: 0x0012,
//             inquiry_scan_type: ScanType::Standard,
//             inquiry_mode: InquiryMode::Standard,
//             page_timeout: 0x2000,
//             connection_accept_timeout: 0x1F40,
//             page_scan_interval: 0x0800,
//             page_scan_window: 0x0012,
//             page_scan_type: ScanType::Standard,
//             voice_setting: 0x00,
//             pin_type: PinType::Variable,
//             failed_contact_counter: 0x0000,
//             authentication_enable: AuthenticationEnable::NotRequired,
//             hold_mode_activity: HoldModeActivity::PageScan,
//             link_policy_settings: 0x00,
//             flush_timeout: 0x0001,
//             num_broadcast_retransmissions: 0x00,
//             link_supervision_timeout: 0x7D00,
//             synchronous_flow_control_enable: false,
//             errorneous_data_reporting: false,
//             class_of_device: 0x00000000,
//             supported_commands: [0; 64],
//             flow_control_mode: FlowControlMode::PacketBased,
//             le_supported_host: false,
//             sync_train_interval: 0x0080,
//             sync_train_timeout: 0x2EE00,
//             service_data: 0x00,
//             secure_connections_host_support: false,
//             authentication_payload_timeout: 0x0001,
//             extended_page_timeout: 0x0000,
//             extended_inquiry_length: 0x0000,
//         }
//     }
// }

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum PageScanRepetitionMode {
    R0 = 0,
    R1,
    R2,
}

#[repr(u8)]
pub enum ScanEnable {
    NoScansEnable,
    InquiryEnablePageDisable,
    InquiryDisablePageEnable,
    InquiryEnablePageEnable,
}

#[derive(EnumU8ToLeBytes, Clone)]
#[repr(u8)]
pub enum LEAddressType {
    PublicDevice,
    RandomDevice,
    PublicIdentity,
    RandomIdentity,
}

#[derive(EnumU8ToLeBytes, Clone)]
#[repr(u8)]
pub enum LEAddressType2 {
    PublicDeviceOrPublicIdentity,
    RandomDeviceOrRandomIdentity,
}

impl From<LEAddressType> for LEAddressType2 {
    fn from(value: LEAddressType) -> Self {
        match value {
            LEAddressType::PublicDevice => LEAddressType2::PublicDeviceOrPublicIdentity,
            LEAddressType::RandomDevice => LEAddressType2::RandomDeviceOrRandomIdentity,
            LEAddressType::PublicIdentity => LEAddressType2::PublicDeviceOrPublicIdentity,
            LEAddressType::RandomIdentity => LEAddressType2::RandomDeviceOrRandomIdentity,
        }
    }
}

#[derive(EnumU8ToLeBytes, Clone)]
#[repr(u8)]
pub enum AdvertisingType {
    ConnectableAndScannnable,
    ConnectableHighDuty,
    Scannable,
    NonConnectable,
    ConnectableLowDuty,
}

#[derive(EnumU8ToLeBytes, Clone)]
#[repr(u8)]
pub enum AdvertisingFilterPolicy {
    UnFilter,
    FilterOnlyScan,
    FilterOnlyConnect,
    FilterBoth,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum ScanType {
    /// Mandatory Range
    Standard,
    /// Optional
    Interlaced,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum InquiryMode {
    Standard,
    WithRSSI,
    WithRSSIAndExtended,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum PinType {
    Variable,
    Fixed,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum AuthenticationEnable {
    NotRequired,
    Required,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum HoldModeActivity {
    PageScan,
    InquiryScan,
    PeriodicInquiries,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum FlowControlMode {
    PacketBased,
    DataBlockBased,
}

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum BDAddrType {
    LEPublic,
    LERandom,
    Classic,
}

bitflags! {
    #[derive(PartialEq)]
    pub struct LEAdvertisementsState: u8 {
        const Idle = 0;
        const Active = 0x01;
        const Enabled = 0x02;
    }
}

bitflags! {
    #[derive(PartialEq)]
    pub struct LEAdvertisementsTodo: u16 {
        const Idle = 0;
        const SetAdvData = 1 << 0;
        const SetScanData = 1 << 1;
        const SetParams = 1 << 2;
        const SetPeriodicParams = 1 << 3;
        const SetPeriodicData = 1 << 4;
        const RemoveSet = 1 << 5;
        const SetAddress = 1 << 6;
        const SetAddressSet0 = 1 << 7;
        const PrivacyNotify = 1 << 8;
    }
}

use crate::host::hci::HCI;
pub trait HCICmdSend {
    fn send(&self, hci: &mut HCI);
}

// pub trait DiscEnum<T> {
//     fn discriminant(&self) -> T;
// }
