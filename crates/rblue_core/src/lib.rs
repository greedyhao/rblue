#![no_std]
pub mod hci;
extern crate alloc;

pub type BDAddr = [u8; 6];

pub enum BtCmd {
    Test,
}

impl BtCmd {
    pub fn exec(&self) {}
}
