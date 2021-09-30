use anyhow::{anyhow, Result};
use clap::{App, Arg};
use env_logger;
use hex::decode;
use rustymind::{connect_headset, AsicEeg, PacketType, Parser, HEADSETID_AUTOCONNECT};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct Train {
    pub attention: u8,
    pub meditation: u8,
    pub poor_signal: u8,
    pub raw_val: Vec<i16>,
    pub eeg: AsicEeg,
}

impl Train {
    fn new() -> Train {
        Train {
            attention: 0,
            meditation: 0,
            poor_signal: 0,
            raw_val: Vec::new(),
            eeg: AsicEeg::new(),
        }
    }

    fn write<W: Write>(&mut self, mut writer: W) -> Result<()> {
        let mut j = serde_json::to_string(&self)?;
        j.push_str(&"\n");
        writer.write_all(&j.as_bytes())?;
        *self = Train::new();
        Ok(())
    }
}

#[allow(unreachable_code)]
fn main() -> Result<()> {
    env_logger::init();
    let matches = App::new("rustymind")
        .version("1.0")
        .author("Junjun Dong <junjun.dong9@gmail.com>")
        .about("parse mindwaves and draw real time plots")
        .arg(
            Arg::with_name("dongle-path")
                .help("Sets the dongle path")
                .required(true),
        )
        .arg(Arg::with_name("HEADSET_ID").help(
            "Sets the headset ID. Set headset ID to 0xc2 to switch into auto-connect mode and connect to any to any headsets dongle can find",
        ))
        .get_matches();
    let headset = matches
        .value_of("HEADSET_ID")
        .map_or(HEADSETID_AUTOCONNECT.to_vec(), |v| {
            decode(v).expect("Hex decoding failed")
        });
    let path = matches.value_of("dongle-path").unwrap();
    let mut port = connect_headset(path, &headset[..]).map_err(|s| anyhow!(s))?;
    let mut read_buf: Vec<u8> = vec![0; 2048];
    let mut parser = Parser::new();
    let mut buffer = File::create("train_data.json")?;
    let mut train_data = Train::new();

    loop {
        let bytes_read = port.read(read_buf.as_mut_slice()).expect(
            "Found no data when reading from dongle. Please make sure headset is connected.",
        );
        for i in 0..bytes_read {
            if let Some(x) = parser.parse(read_buf[i]) {
                for r in x {
                    match r {
                        PacketType::RawValue(value) => {
                            train_data.raw_val.push(value);
                        }
                        PacketType::PoorSignal(value) => {
                            train_data.poor_signal = value;
                        }
                        PacketType::AsicEeg(value) => {
                            train_data.eeg = value;
                        }
                        PacketType::Attention(value) => {
                            train_data.attention = value;
                        }
                        PacketType::Meditation(value) => {
                            train_data.meditation = value;
                            train_data.write(&mut buffer)?;
                        }
                        PacketType::PacketUndefined(value) => {
                            println!("undefinded value = {}", value);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(())
}
