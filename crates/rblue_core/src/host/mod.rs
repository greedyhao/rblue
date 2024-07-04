pub mod hci;
pub mod hci_cmd;

pub use crate::BDAddr;
pub use hci::HCICmd;

pub use hci::LinkControl;
// pub use hci::LinkPolicy;
pub use hci::ControllerAndBaseband;
pub use hci::InformationalParam;
// pub use hci::StatusParam;
// pub use hci::TestingCommand;
pub use hci::LEController;

use bitflags::bitflags;
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
pub enum PageScanRepetitionMode {
    R0 = 0,
    R1,
    R2,
}

#[derive(serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum ScanEnable {
    NoScansEnable,
    InquiryEnablePageDisable,
    InquiryDisablePageEnable,
    InquiryEnablePageEnable,
}

#[derive(serde_repr::Serialize_repr)]
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
