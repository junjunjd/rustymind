use rustymind::connect_headset;
use rustymind::Parser;

fn main() {
    env_logger::init();
    let headset = [0xa2, 0x6c];
    let mut port = connect_headset(&headset);
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
}
