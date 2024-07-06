#![no_std]
pub mod host;
pub mod baseband;

extern crate alloc;

pub type BDAddr = [u8; 6];


#[cfg(test)]
mod tests {
}
