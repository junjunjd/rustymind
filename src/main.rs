use clap::{App, Arg};
use env_logger;
use hex::decode;
use rustymind::{connect_headset, Parser, HEADSETID_AUTOCONNECT};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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

    println!(
        "Using dongle path: {}",
        matches.value_of("dongle-path").unwrap()
    );
    println!(
        "Using headset ID: {}",
        matches.value_of("HEADSET_ID").unwrap()
    );
    env_logger::init();
    let headset = matches
        .value_of("HEADSET_ID")
        .map_or(HEADSETID_AUTOCONNECT.to_vec(), |v| {
            decode(v).expect("Hex decoding failed")
        });
    let path = matches.value_of("dongle-path").unwrap();
    let mut port = connect_headset(path, &headset[..])?;
    let mut temp: Vec<u8> = vec![0];
    let mut parser = Parser::new();
    let mut result_vec = Vec::new();

    loop {
        port.read(temp.as_mut_slice())
            .expect("Found no data when reading from connect_headset!");
        if let Some(x) = parser.parse(temp[0]) {
            result_vec.push(x);
        }
    }
    Ok(())
}
