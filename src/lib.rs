use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

pub const HEADSETID_AUTOCONNECT: [u8; 1] = [0xc2];

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
    AsicEeg(AsicEeg),
    PacketUndefined(u8),
}

pub enum State {
    NoSync,
    FirstSync,
    SecondSync,
    ValidPacket,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AsicEeg {
    pub delta: u32,
    pub theta: u32,
    pub low_alpha: u32,
    pub high_alpha: u32,
    pub low_beta: u32,
    pub high_beta: u32,
    pub low_gamma: u32,
    pub mid_gamma: u32,
}

impl AsicEeg {
    pub fn new() -> AsicEeg {
        AsicEeg {
            delta: 0,
            theta: 0,
            low_alpha: 0,
            high_alpha: 0,
            low_beta: 0,
            high_beta: 0,
            low_gamma: 0,
            mid_gamma: 0,
        }
    }
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
            debug!("Standby for a valid packet");
        }
    }

    fn handle_firstsync(&mut self, data: u8) {
        if data == 0xaa {
            self.state = State::SecondSync;
            debug!("Packet synced");
        } else {
            self.state = State::NoSync;
        }
    }

    fn handle_secondsync(&mut self, data: u8) {
        if data > 0xaa {
            self.state = State::NoSync;
            error!("Plength larger than 170!");
        } else if data < 0xaa {
            self.state = State::ValidPacket;
            self.plength = data;
            debug!("Valid packet available, len({})", self.plength);
        }
    }

    fn handle_validpacket(&mut self, data: u8) -> Option<Vec<PacketType>> {
        if self.plength == 0 {
            self.checksum = !self.checksum;
            let re = if data != self.checksum {
                debug!("Checksum failed");
                None
            } else {
                debug!("Checksum matched, start parsing");
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
                    info!(
                        "headset connected, ID {:#04x} {:#04x}",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetConnected(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                } else {
                    warn!("undefined packet while headset connected");
                    result.push(PacketType::HeadsetConnectedUndefined);
                }

                n += 4;
            } else if self.payload[n] == 0xd1 {
                // Headset Not Found
                if self.payload[n + 1] == 0x02 {
                    warn!(
                        "Headset {:#04x} {:#04x} not found",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetNotFound(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                    n += 4;
                } else if self.payload[n + 1] == 0x00 {
                    warn!("no headset could be found during Connect All.");
                    result.push(PacketType::NoHeadsetFound);
                    n += 2;
                } else {
                    warn!("undefined packetLength while headset not found");
                    result.push(PacketType::NotFoundUndefined);
                }
            } else if self.payload[n] == 0xd2 {
                if self.payload[n + 1] == 0x02 {
                    info!(
                        "disconnected from headset {:#04x} {:#04x}",
                        self.payload[n + 2],
                        self.payload[n + 3]
                    );
                    result.push(PacketType::HeadsetDisconnected(
                        ((self.payload[n + 2] as u16) << 8) | (self.payload[n + 3] as u16),
                    ));
                } else {
                    warn!("undefined packetLength while headset disconnected");
                    result.push(PacketType::HeadsetDisconnectedUndefined);
                }
                n += 4;
            } else if self.payload[n] == 0xd3 {
                if self.payload[n + 1] == 0x00 {
                    warn!("the last command request was denied");
                    result.push(PacketType::RequestDenied);
                } else {
                    warn!("undefined packetLength while headset disconnected");
                    result.push(PacketType::HeadsetDisconnectedUndefined);
                }
                n += 2;
            } else if self.payload[n] == 0xd4 {
                if self.payload[n + 1] == 0x01 {
                    if self.payload[n + 2] == 0x00 {
                        debug!("headset is in standby mode awaiting for a command");
                        result.push(PacketType::Standby);
                    } else if self.payload[n + 2] == 0x01 {
                        debug!("connecting to a headset");
                        result.push(PacketType::FindHeadset);
                    } else {
                        warn!("undefined packet code while standby");
                        result.push(PacketType::StandbyPacketUndefined);
                    }
                } else {
                    warn!("undefined packet length while standby");
                    result.push(PacketType::StandbyLengthUndefined);
                }
                n += 3;
            } else if self.payload[n] == 0x02 {
                // poor signal
                if self.payload[n + 1] == 200 {
                    warn!("the ThinkGear contacts are not touching the user's skin");
                } else {
                    debug!("Poor signal quality {:#04x}", self.payload[n + 1]);
                }
                result.push(PacketType::PoorSignal(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x04 {
                // attention
                debug!("Attention esense {:#04x}", self.payload[n + 1]);
                result.push(PacketType::Attention(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x05 {
                // meditation
                debug!("Meditation esense {:#04x}", self.payload[n + 1]);
                result.push(PacketType::Meditation(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x16 {
                // blink
                debug!("Blink strength {:#04x}", self.payload[n + 1]);
                result.push(PacketType::Blink(self.payload[n + 1]));
                n += 2;
            } else if self.payload[n] == 0x80 {
                // RAW Wave Value: a single big-endian 16-bit two's-compliment signed value
                // (high-order byte followed by low-order byte) (-32768 to 32767)
                let raw_val: i16 =
                    ((self.payload[n + 2] as i16) << 8) | (self.payload[n + 3] as i16);
                debug!("Raw value {:#04x}", raw_val);
                result.push(PacketType::RawValue(raw_val));
                n += 4;
            } else if self.payload[n] == 0x83 {
                //ASIC_EEG_POWER: eight big-endian 3-byte unsigned integer values representing
                //delta, theta, low-alpha high-alpha, low-beta, high-beta, low-gamma, and mid-gamma
                //EEG band power values
                let mut eeg_vec: Vec<u32> = vec![];

                for i in 0..8 {
                    let asic = ((self.payload[n + 2 + i * 3] as u32) << 16)
                        | ((self.payload[n + 3 + i * 3] as u32) << 8)
                        | (self.payload[n + 4 + i * 3] as u32);
                    eeg_vec.push(asic);
                }

                let eeg_power = AsicEeg {
                    delta: eeg_vec[0],
                    theta: eeg_vec[1],
                    low_alpha: eeg_vec[2],
                    high_alpha: eeg_vec[3],
                    low_beta: eeg_vec[4],
                    high_beta: eeg_vec[5],
                    low_gamma: eeg_vec[6],
                    mid_gamma: eeg_vec[7],
                };
                debug!("EEG power values = {:?}", eeg_power);
                result.push(PacketType::AsicEeg(eeg_power));
                n += 26;
            } else {
                warn!("packet code undefined {:#04x}", self.payload[n]);
                result.push(PacketType::PacketUndefined(self.payload[n]));
                n += 1;
            }
        }
        debug!("end of packet");
        result
    }
}

pub fn connect_headset(
    path: &str,
    headset: &[u8],
) -> Result<Box<dyn serialport::SerialPort>, &'static str> {
    let mut port = serialport::new(path, 115_200)
        .timeout(core::time::Duration::from_millis(1000))
        .open()
        .map_err(|_| "Cannot connect to dongle. Please make sure the serial number of your dongle is correct.")?;

    const DISCONNECT: u8 = 0xc1;
    const CONNECT: u8 = 0xc0;
    let mut serial_buf: Vec<u8> = vec![0];

    port.write(&[DISCONNECT])
        .map_err(|_| "Failed to write DISCONNECT to dongle.")?;
    port.read(serial_buf.as_mut_slice())
        .map_err(|_| "Cannot read data from dongle.")?;
    if headset.len() != 1 {
        port.write(&[CONNECT])
            .map_err(|_| "Failed to write CONNECT to dongle.")?;
    }
    port.write(&headset)
        .map_err(|_| "Failed to write headset ID to dongle.")?;
    return Ok(port);
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

        let test_asic = AsicEeg {
            delta: 0x94,
            theta: 0x42,
            low_alpha: 0x0b,
            high_alpha: 0x64,
            low_beta: 0x4d,
            high_beta: 0x3d,
            low_gamma: 0x07,
            mid_gamma: 0x05,
        };

        assert_eq!(
            result,
            vec![
                PacketType::PoorSignal(0x00),
                PacketType::AsicEeg(test_asic),
                PacketType::Attention(0x0d),
                PacketType::Meditation(0x3d)
            ]
        );
    }
}
