#[derive(PartialEq, Eq, Debug)]
pub enum PacketType {
    HeadsetConnected(u16),
    HeadsetConnectedUndefined,
    HeadsetNotFound(u16),
    NoHeadsetFound,
    NotFoundUndefined,
    HeadsetDisconnected(u16),
    HeadsetDisconnectedUndefined,
    RequestDenied,
    Standby,
    FindHeadset,
    StandbyPacketUndefined,
    StandbyLengthUndefined,
    PoorSignal(u8),
    Attention(u8),
    Meditation(u8),
    Blink(u8),
    RawValue(i16),
    AsicEgg(Vec<u32>),
    PacketUndefined(u8),
}

pub enum State {
    NoSync,
    FirstSync,
    SecondSync,
    ValidPacket,
}

pub struct Parser {
    state: State,
    plength: u8,
    payload: Vec<u8>,
    checksum: u8,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: State::NoSync,
            plength: 0,
            payload: Vec::new(),
            checksum: 0,
        }
    }
}

impl Parser {
    pub fn parse(&mut self, data: u8) -> Option<Vec<PacketType>> {
        match self.state {
            State::NoSync => {
                self.handle_nosync(data);
                None
            }
            State::FirstSync => {
                self.handle_firstsync(data);
                None
            }
            State::SecondSync => {
                self.handle_secondsync(data);
                None
            }
            State::ValidPacket => self.handle_validpacket(data),
        }
    }

    fn reset(&mut self) {
        *self = Parser::new();
    }

    fn handle_nosync(&mut self, data: u8) {
        if data == 0xaa {
            self.state = State::FirstSync;
            println!("-------- Standby for a valid packet --------");
        }
    }

    fn handle_firstsync(&mut self, data: u8) {
        if data == 0xaa {
            self.state = State::SecondSync;
            println!("-------- Packet synced --------");
        } else {
            self.state = State::NoSync;
        }
    }

    fn handle_secondsync(&mut self, data: u8) {
        if data > 0xaa {
            self.state = State::NoSync;
            println!("********** ERROR : Plength larger than 170! **********");
        } else if data < 0xaa {
            self.state = State::ValidPacket;
            self.plength = data;
            println!(
                "-------- Valid packet available, len({}) --------",
                self.plength
            );
        }
    }

    fn handle_validpacket(&mut self, data: u8) -> Option<Vec<PacketType>> {
        if self.plength == 0 {
            self.checksum = !self.checksum;
            let re = if data != self.checksum {
                println!("********** Checksum failed **********");
                None
            } else {
                println!("---------- checksum matched, start parsing ----------");
                Some(self.handle_parser())
            };
            self.reset();
            re
        } else {
            self.payload.push(data);
            self.checksum = self.checksum.overflowing_add(data).0;
            self.plength -= 1;
            None
        }
    }

    fn handle_parser(&mut self) -> Vec<PacketType> {
        let mut n = 0;
        let mut result: Vec<PacketType> = Vec::new();
        while n < self.payload.len() {
            if self.payload[n] == 0xd0 {
                // Headset Connected
                if self.payload[n + 1] == 0x02 {
                    println!(
                        "----- headset connected, ID {:#04x} {:#04x} -----",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetConnected(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                } else {
                    println!("undefined packet while headset connected");
                    result.push(PacketType::HeadsetConnectedUndefined);
                    // ??? how to handle datarow length
                }

                n += 4;
            } else if self.payload[n] == 0xd1 {
                // Headset Not Found
                if self.payload[n + 1] == 0x02 {
                    println!(
                        "----- Headset ID {:#04x} {:#04x} could not be found -----",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetNotFound(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                    n += 4;
                } else if self.payload[n + 1] == 0x00 {
                    println!("no headset could be found during Connect All.");
                    result.push(PacketType::NoHeadsetFound);
                    n += 2;
                } else {
                    println!("undefined packetLength while headset not found");
                    result.push(PacketType::NotFoundUndefined);
                    // ??? how to handle length
                }
            } else if self.payload[n] == 0xd2 {
                if self.payload[n + 1] == 0x02 {
                    println!(
                        "----- disconnected from headset with ID {:#04x} {:#04x} -----",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetDisconnected(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                } else {
                    println!("undefined packetLength while headset disconnected");
                    result.push(PacketType::HeadsetDisconnectedUndefined);
                }
                n += 4;
            } else if self.payload[n] == 0xd3 {
                if self.payload[n + 1] == 0x00 {
                    println!("----- the last command request was denied -----");
                    result.push(PacketType::RequestDenied);
                } else {
                    println!("undefined packetLength while headset disconnected");
                    result.push(PacketType::HeadsetDisconnectedUndefined);
                }
                n += 2;
            } else if self.payload[n] == 0xd4 {
                if self.payload[n + 1] == 0x01 {
                    if self.payload[n + 2] == 0x00 {
                        println!("----- headset is in standby mode awaiting for a command -----");
                        result.push(PacketType::Standby);
                    } else if self.payload[n + 2] == 0x01 {
                        println!("----- dongle is trying to connect to a headset -----");
                        result.push(PacketType::FindHeadset);
                    } else {
                        println!("----- undefined packet code while standby -----");
                        result.push(PacketType::StandbyPacketUndefined);
                    }
                } else {
                    println!("----- undefined packet length while standby -----");
                    result.push(PacketType::StandbyLengthUndefined);
                }
                n += 3;
            } else if self.payload[n] == 0x02 {
                // poor signal
                println!(
                    "========== Poor signal, quality {:#04x} ==========",
                    self.payload[n + 1]
                );
                result.push(PacketType::PoorSignal(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x04 {
                // attention
                println!(
                    "========== Attention, esense {:#04x} ==========",
                    self.payload[n + 1]
                );
                result.push(PacketType::Attention(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x05 {
                // meditation
                println!(
                    "========== Meditation, esense {:#04x} ==========",
                    self.payload[n + 1]
                );
                result.push(PacketType::Meditation(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x16 {
                // blink
                println!(
                    "========== Blink, strength {:#04x} ==========",
                    self.payload[n + 1]
                );
                result.push(PacketType::Blink(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x80 {
                // RAW Wave Value: a single big-endian 16-bit two's-compliment signed value
                // (high-order byte followed by low-order byte) (-32768 to 32767)
                let raw_val: i16 =
                    ((self.payload[n + 2] as i16) << 8) | (self.payload[n + 3] as i16);
                println!("========== Raw value {:#04x} ==========", raw_val);
                result.push(PacketType::RawValue(raw_val));
                n += 4;
            } else if self.payload[n] == 0x83 {
                //ASIC_EEG_POWER: eight big-endian 3-byte unsigned integer values representing
                //delta, theta, low-alpha high-alpha, low-beta, high-beta, low-gamma, and mid-gamma
                //EEG band power values
                let mut current_vec: Vec<u32> = vec![];
                let mut asic: u32 = 0;

                for i in 0..8 {
                    asic = ((self.payload[n + 2 + i * 3] as u32) << 16)
                        | ((self.payload[n + 3 + i * 3] as u32) << 8)
                        | (self.payload[n + 4 + i * 3] as u32);
                    current_vec.push(asic);
                }
                println!("========== Delta {:#04x} ==========", current_vec[0]);
                println!("========== Theta {:#04x} ==========", current_vec[1]);
                println!("========== LowAlpha {:#04x} ==========", current_vec[2]);
                println!("========== HighAlpha {:#04x} ==========", current_vec[3]);
                println!("========== LowBeta {:#04x} ==========", current_vec[4]);
                println!("========== highBeta {:#04x} ==========", current_vec[5]);
                println!("========== LowGamma {:#04x} ==========", current_vec[6]);
                println!("========== midGamma {:#04x} ==========", current_vec[7]);
                result.push(PacketType::AsicEgg(current_vec));
                n += 26;
            } else {
                println!(
                    "********** packet code undefined, {:#04x} **********",
                    self.payload[n]
                );
                result.push(PacketType::PacketUndefined(self.payload[n]));
                n += 1;
            }
        }
        println!("-------- end of packet --------");
        result
    }
}

pub fn dongle() -> Box<dyn serialport::SerialPort> {
    let mut port = serialport::new("/dev/tty.usbserial-14140", 115_200)
        .timeout(core::time::Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let DISCONNECT: u8 = 0xc1;
    let CONNECT: u8 = 0xc0;
    let auto_connect: u8 = 0xc2;
    let headset = [0xa2, 0x6c];
    let mut serial_buf: Vec<u8> = vec![0];

    port.write(&[DISCONNECT]).expect("Write DISCONNECT failed!");
    port.read(serial_buf.as_mut_slice())
        .expect("Started. Found no data!");
    port.write(&[CONNECT]).expect("Write CONNECT failed!");
    port.write(&headset).expect("Write headset ID failed!");
    return port;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parser() {
        let test_vec: Vec<u8> = vec![
            0xAA, // [SYNC]
            0xAA, // [SYNC]
            0x20, // [PLENGTH] (payload length) of 32 bytes
            0x02, // [POOR_SIGNAL] Quality
            0x00, // No poor signal detected (0/200)
            0x83, // [ASIC_EEG_POWER_INT]
            0x18, // [VLENGTH] 24 bytes
            0x00, // (1/3) Begin Delta bytes
            0x00, // (2/3)
            0x94, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x42, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x0B, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x64, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x4D, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x3D, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x07, // (3/3)
            0x00, // (1/3)
            0x00, // (2/3)
            0x05, // (3/3)
            0x04, // [ATTENTION] eSense
            0x0D, // eSense Attention level of 13
            0x05, // [MEDITATION] eSense
            0x3D, // eSense Meditation level of 61
            0x34, // [CHKSUM] (1's comp inverse of 8-bit Payload sum of 0xCB)
        ];
        let mut result: Vec<PacketType> = Vec::new();
        let mut parser = Parser::new();
        for data in test_vec {
            if let Some(x) = parser.parse(data) {
                result = x;
            }
        }

        let test_asic: Vec<u32> = vec![0x94, 0x42, 0x0b, 0x64, 0x4d, 0x3d, 0x07, 0x05];
        assert_eq!(
            result,
            vec![
                PacketType::PoorSignal(0x00),
                PacketType::AsicEgg(test_asic),
                PacketType::Attention(0x0d),
                PacketType::Meditation(0x3d)
            ]
        );
    }
}
