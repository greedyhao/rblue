use crate::baseband::Control;
use crate::baseband::ControllerErrorCode;

use log::info;

use crate::host::hci::HCIEvent;
use crate::host::hci_cmd::*;

macro_rules! create_hci_cmd_table {
    ($num:expr, $bit:expr, $handler:ident) => {
        Some(HCICmdTable {
            flag: compute_hci_cmd_flag($num, $bit),
            handle: $handler,
        })
    };
}

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
const HCI_SET_EVENT_MASK_BIT: u8 = 0x40;
const HCI_RESET_BIT: u8 = 0x80;

// byte14
// const HCI_READ_LOCAL_VERSION_INFORMATION_BIT: u8 = 0x08;
const HCI_READ_LOCAL_SUPPORTED_COMMANDS_BIT: u8 = 0x10;
const HCI_READ_LOCAL_SUPPORTED_FEATURES_BIT: u8 = 0x20;
// const HCI_READ_LOCAL_EXTENDED_FEATURES_BIT: u8 = 0x40;
const HCI_READ_BUFFER_SIZE_BIT: u8 = 0x80;

// byte15
const HCI_READ_BD_ADDR_BIT: u8 = 0x02;

// byte25
const HCI_LE_SET_EVENT_MASK_BIT: u8 = 0x01;
const HCI_LE_READ_BUFFER_SIZE_BIT: u8 = 0x02;
const HCI_LE_READ_LOCAL_SUPPORTED_FEATURES_BIT: u8 = 0x04;
// const HCI_LE_SET_RANDOM_ADDRESS_BIT: u8 = 0x10;
const HCI_LE_SET_ADVERTISING_PARAMETERS_BIT: u8 = 0x20;
const HCI_LE_READ_ADVERTISING_PHYSICAL_CHANNEL_TX_POWER_BIT: u8 = 0x40;
const HCI_LE_SET_ADVERTISING_DATA_BIT: u8 = 0x80;

// byte26
const HCI_LE_SET_SCAN_RESPONSE_DATA_BIT: u8 = 0x01;
const HCI_LE_SET_ADVERTISING_ENABLE_BIT: u8 = 0x02;
// const HCI_LE_SET_SCAN_PARAMETERS_BIT: u8 = 0x04;
// const HCI_LE_SET_SCAN_ENABLE_BIT: u8 = 0x08;
// const HCI_LE_CREATE_CONNECTION_BIT: u8 = 0x10;
// const HCI_LE_CREATE_CONNECTION_CANCEL_BIT: u8 = 0x20;
// const HCI_LE_READ_FILTER_ACCEPT_LIST_SIZE_BIT: u8 = 0x40;
// const HCI_LE_CLEAR_FILTER_ACCEPT_LIST_BIT: u8 = 0x80;

const TABLE_LINK_CONTROL: &[Option<HCICmdTable>] = &[];
const TABLE_LINK_POLICY: &[Option<HCICmdTable>] = &[];
const TABLE_CONTROLLER_AND_BASEBAND: &[Option<HCICmdTable>] = &[
    create_hci_cmd_table!(5, HCI_SET_EVENT_MASK_BIT, set_event_mask),
    None,
    create_hci_cmd_table!(5, HCI_RESET_BIT, reset),
];
const TABLE_INFORMATIONAL_PARAM: &[Option<HCICmdTable>] = &[
    None,
    create_hci_cmd_table!(
        14,
        HCI_READ_LOCAL_SUPPORTED_COMMANDS_BIT,
        read_local_supported_commands
    ),
    create_hci_cmd_table!(
        14,
        HCI_READ_LOCAL_SUPPORTED_FEATURES_BIT,
        read_local_supported_features
    ),
    None,
    create_hci_cmd_table!(14, HCI_READ_BUFFER_SIZE_BIT, read_buffer_size),
    None,
    None,
    None,
    create_hci_cmd_table!(15, HCI_READ_BD_ADDR_BIT, read_bd_address),
];
const TABLE_STATUS_PARAM: &[Option<HCICmdTable>] = &[];
const TABLE_TESTING_COMMAND: &[Option<HCICmdTable>] = &[];
const TABLE_REVERSE: &[Option<HCICmdTable>] = &[];
const TABLE_LE_CONTROLLER: &[Option<HCICmdTable>] = &[
    create_hci_cmd_table!(25, HCI_LE_SET_EVENT_MASK_BIT, le_set_event_mask),
    create_hci_cmd_table!(25, HCI_LE_READ_BUFFER_SIZE_BIT, le_read_buffer_size),
    create_hci_cmd_table!(
        25,
        HCI_LE_READ_LOCAL_SUPPORTED_FEATURES_BIT,
        le_read_local_supported_features
    ),
    None,
    None,
    create_hci_cmd_table!(
        25,
        HCI_LE_SET_ADVERTISING_PARAMETERS_BIT,
        le_set_advertising_parameters
    ),
    create_hci_cmd_table!(
        25,
        HCI_LE_READ_ADVERTISING_PHYSICAL_CHANNEL_TX_POWER_BIT,
        le_read_advertising_physical_channel_tx_power
    ),
    create_hci_cmd_table!(25, HCI_LE_SET_ADVERTISING_DATA_BIT, le_set_advertising_data),
    create_hci_cmd_table!(
        26,
        HCI_LE_SET_SCAN_RESPONSE_DATA_BIT,
        le_set_scan_response_data
    ),
    create_hci_cmd_table!(
        26,
        HCI_LE_SET_ADVERTISING_ENABLE_BIT,
        le_set_advertising_enable
    ),
];

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

const HCI_CMD_SUPPORTED_BYTES: [u8; 64] = compute_hci_cmd_support(HCI_CMD_TABLE);

const LMP_SUPPORTED_FEATURES_BYTES: [u8; 8] = [0; 8];

fn bb_send_event<T>(bb: &mut Control, opcode: u16, ret: T)
where
    T: RBlueToU8Array,
{
    let evt = CommandCompleteEvt {
        num_hci_command_packets: 5,
        opcode,
        return_param: ret,
    };
    bb.send_event(HCIEvent::CommandComplete as u8, evt.to_u8_array());
}

// Controller and Baseband Commands

fn set_event_mask(bb: &mut Control, opcode: u16) {
    let ret = SetEventMaskRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

fn reset(bb: &mut Control, opcode: u16) {
    info!("bb reset");
    // reset
    bb.power_on();

    let ret = ResetRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

// Informational Parameters

fn read_local_supported_commands(bb: &mut Control, opcode: u16) {
    let ret = ReadLocalSupportedCommandsRet {
        status: ControllerErrorCode::Ok,
        supported_commands: HCI_CMD_SUPPORTED_BYTES,
    };

    bb_send_event(bb, opcode, ret);
}

fn read_local_supported_features(bb: &mut Control, opcode: u16) {
    let ret = ReadLocalSupportedFeaturesRet {
        status: ControllerErrorCode::Ok,
        lmp_feature: LMP_SUPPORTED_FEATURES_BYTES,
    };

    bb_send_event(bb, opcode, ret);
}

fn read_buffer_size(bb: &mut Control, opcode: u16) {
    let ret = ReadBufferSizeRet {
        status: ControllerErrorCode::Ok,
        acl_data_packet_length: 0,
        synchronous_data_packet_length: 0,
        total_num_acl_data_packets: 0,
        total_num_synchronous_data_packets: 0,
    };

    bb_send_event(bb, opcode, ret);
}

fn read_bd_address(bb: &mut Control, opcode: u16) {
    let ret = ReadBDAddrRet {
        status: ControllerErrorCode::Ok,
        bd_addr: [0; 6],
    };

    bb_send_event(bb, opcode, ret);
}

// LE Controller Commands

fn le_set_event_mask(bb: &mut Control, opcode: u16) {
    let ret = LeSetEventMaskRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_read_buffer_size(bb: &mut Control, opcode: u16) {
    let ret = LEReadBufferSizeRet {
        status: ControllerErrorCode::Ok,
        le_acl_data_packet_length: 0,
        total_num_le_acl_data_packets: 0,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_read_local_supported_features(bb: &mut Control, opcode: u16) {
    let ret = LEReadLocalSupportedFeaturesRet {
        status: ControllerErrorCode::Ok,
        le_features: 0,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_set_advertising_parameters(bb: &mut Control, opcode: u16) {
    let ret = LESetAdvertisingParametersRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_read_advertising_physical_channel_tx_power(bb: &mut Control, opcode: u16) {
    let ret = LEReadAdvertisingPhysicalChannelTxPowerRet {
        status: ControllerErrorCode::Ok,
        tx_power_level: 0,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_set_advertising_data(bb: &mut Control, opcode: u16) {
    let ret = LESetAdvertisingDataRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_set_scan_response_data(bb: &mut Control, opcode: u16) {
    let ret = LESetScanResponseDataRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}

fn le_set_advertising_enable(bb: &mut Control, opcode: u16) {
    let ret = LESetAdvertisingEnableRet {
        status: ControllerErrorCode::Ok,
    };

    bb_send_event(bb, opcode, ret);
}
