pub enum Error {
    Ok,
}

pub enum PacketType {
    Unknown,
    Command,
    ACLData,
    SCOData,
    Event,
    ISOData,
}

pub type PacketHandler = fn (PacketType, &[u8]);

pub trait HciDevice {
    fn open(&self) -> Result<(), Error>;
    fn close(&self) -> Result<(), Error>;
    fn register_packet_handler(&mut self, handler: PacketHandler);
    fn can_send_packet_now(&self, packet_type: PacketType) -> bool;
    fn send_packet(&self, packet_type: PacketType, packet: &[u8]) -> Result<(), Error>;
    fn recv_packet(&mut self, packet: &mut [u8]);
    fn set_baudrate(&self, baudrate: u32) -> Result<(), Error>;
}
