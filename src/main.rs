use clap::{App, Arg};
use env_logger;
use hex::decode;
use rustymind::{connect_headset, PacketType, Parser, HEADSETID_AUTOCONNECT};
use std::error::Error;

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

    loop {
        let bytes_read = port.read(read_buf.as_mut_slice()).expect(
            "Found no data when reading from dongle. Please make sure headset is connected.",
        );
        for i in 0..bytes_read {
            if let Some(x) = parser.parse(read_buf[i]) {
                for r in x {
                    match r {
                        PacketType::Attention(value) => {
                            println!("Attention value = {}", value);
                        }
                        PacketType::Meditation(value) => {
                            println!("Meditation value = {}", value);
                        }
                        PacketType::AsicEeg(value) => {
                            println!("EEG power values = {:?}", value);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(())
}
