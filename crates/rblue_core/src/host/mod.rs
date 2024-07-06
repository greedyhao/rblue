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

#[derive(EnumU8ToLeBytes)]
#[repr(u8)]
pub enum LEAddressType {
    PublicDevice,
    RandomDevice,
    PublicIdentity,
    RandomIdentity,
}

use crate::host::hci::HCI;
pub trait HCICmdSend {
    fn send(&self, hci: &mut HCI);
}

// pub trait DiscEnum<T> {
//     fn discriminant(&self) -> T;
// }
