//fn main() -> Result<(), Box<dyn Error>> {
//  let mut wtr = Writer::from_path("foo.csv")?;
//  wtr.write_record(&[1, 2, 3])?;
//  wtr.write_record(&["a", "b", "c"])?;
//  wtr.write_record(&["x", "y", "z"])?;
//  wtr.flush()?;
//  Ok(())
//}
use clap::{App, Arg};
use csv::Writer;
use env_logger;
use hex::decode;
use rustymind::{connect_headset, AsicEeg, PacketType, Parser, HEADSETID_AUTOCONNECT};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct Train {
    attention: u8,
    meditation: u8,
    poor_signal: u8,
    raw_val: Vec<i16>,
    eeg: AsicEeg,
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
}

#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn Error>> {
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
    let mut port = connect_headset(path, &headset[..])?;
    let mut read_buf: Vec<u8> = vec![0; 2048];
    let mut parser = Parser::new();
    let eeg_power = AsicEeg::new();
    //let mut wtr = Writer::from_path("train.csv")?;
    let mut train_data = Train {
        attention: 0,
        meditation: 0,
        poor_signal: 0,
        raw_val: Vec::new(),
        eeg: eeg_power,
    };

    loop {
        let bytes_read = port.read(read_buf.as_mut_slice()).expect(
            "Found no data when reading from dongle. Please make sure headset is connected.",
        );
        for i in 0..bytes_read {
            if let Some(x) = parser.parse(read_buf[i]) {
                let mut raw: Vec<i32> = Vec::new();
                for r in x {
                    match r {
                        PacketType::RawValue(value) => {
                            println!("Raw value = {}", value);
                            train_data.raw_val.push(value);
                        }
                        PacketType::PoorSignal(value) => {
                            println!("Poor signal value = {}", value);
                            train_data.poor_signal = value;
                        }
                        PacketType::AsicEeg(value) => {
                            println!("EEG power values = {:?}", value);
                            train_data.eeg = value;
                        }
                        PacketType::Attention(value) => {
                            println!("Attention value = {}", value);
                            train_data.attention = value;
                        }
                        PacketType::Meditation(value) => {
                            println!("Meditation value = {}", value);
                            train_data.meditation = value;
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
