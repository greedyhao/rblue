use crate::baseband::Control;
use crate::baseband::ControllerErrorCode;

use alloc::vec;
use log::info;

use crate::host::hci::{CommandCompleteArg, HCIEvent};

pub struct HCICmdTable {
    flag: u16,
    pub handle: fn(bb: &mut Control, opcode: u16),
}

const fn compute_hci_cmd_flag(byten: u8, bit: u8) -> u16 {
    ((byten as u16) << 8) | (bit as u16)
}

// byte0
// const HCI_INQUIRY_BIT: u8 = 0x01;
// const HCI_INQUIRY_CANCEL_BIT: u8 = 0x02;

// byte5
const HCI_RESET_BIT: u8 = 0x80;

// byte14
// const HCI_READ_LOCAL_VERSION_INFORMATION_BIT: u8 = 0x08;
const HCI_READ_LOCAL_SUPPORTED_COMMANDS_BIT: u8 = 0x10;
// const HCI_READ_LOCAL_SUPPORTED_FEATURES_BIT: u8 = 0x20;
// const HCI_READ_LOCAL_EXTENDED_FEATURES_BIT: u8 = 0x40;
// const HCI_READ_BUFFER_SIZE_BIT: u8 = 0x80;

const TABLE_LINK_CONTROL: &[Option<HCICmdTable>] = &[];
const TABLE_LINK_POLICY: &[Option<HCICmdTable>] = &[];
const TABLE_CONTROLLER_AND_BASEBAND: &[Option<HCICmdTable>] = &[
    None,
    None,
    Some(HCICmdTable {
        flag: compute_hci_cmd_flag(5, HCI_RESET_BIT),
        handle: reset,
    }),
];
const TABLE_INFORMATIONAL_PARAM: &[Option<HCICmdTable>] = &[
    None,
    Some(HCICmdTable {
        flag: compute_hci_cmd_flag(14, HCI_READ_LOCAL_SUPPORTED_COMMANDS_BIT),
        handle: read_local_supported_commands,
    }),
];
const TABLE_STATUS_PARAM: &[Option<HCICmdTable>] = &[];
const TABLE_TESTING_COMMAND: &[Option<HCICmdTable>] = &[];
const TABLE_REVERSE: &[Option<HCICmdTable>] = &[];
const TABLE_LE_CONTROLLER: &[Option<HCICmdTable>] = &[];

pub const HCI_CMD_TABLE: &[&[Option<HCICmdTable>]; 8] = &[
    TABLE_LINK_CONTROL,
    TABLE_LINK_POLICY,
    TABLE_CONTROLLER_AND_BASEBAND,
    TABLE_INFORMATIONAL_PARAM,
    TABLE_STATUS_PARAM,
    TABLE_TESTING_COMMAND,
    TABLE_REVERSE,
    TABLE_LE_CONTROLLER,
];

const fn compute_hci_cmd_support(table: &[&[Option<HCICmdTable>]; 8]) -> [u8; 64] {
    let mut support = [0; 64];
    let mut i = 0;
    while i < table.len() {
        let sub = table[i];
        let mut j = 0;
        while j < sub.len() {
            if let Some(cmd) = &sub[j] {
                let byte = cmd.flag >> 8;
                let bit = cmd.flag & 0xff;
                support[byte as usize] |= bit as u8;
            }
            j += 1;
        }
        i += 1;
    }
    support
}

const HCI_CMD_SUPPORT_BYTES: [u8; 64] = compute_hci_cmd_support(HCI_CMD_TABLE);

fn reset(bb: &mut Control, opcode: u16) {
    info!("bb reset");
    // reset
    bb.power_on();

    let evt = CommandCompleteArg {
        num_hci_command_packets: 5,
        opcode,
        return_param: ControllerErrorCode::Ok,
    };

    bb.send_event(
        HCIEvent::CommandComplete as u8,
        bincode::serialize(&evt).unwrap(),
    );
}

fn read_local_supported_commands(bb: &mut Control, _opcode: u16) {
    let mut tmp = vec![ControllerErrorCode::Ok as u8];
    tmp.extend(HCI_CMD_SUPPORT_BYTES);
    bb.send_event(HCIEvent::CommandComplete as u8, tmp);
}
