use rustymind::dongle;
use rustymind::Parser;

fn main() {
    let mut port = dongle();
    let mut temp: Vec<u8> = vec![0];
    let mut parser = Parser::new();
    let mut result_vec = Vec::new();

    loop {
        port.read(temp.as_mut_slice())
            .expect("Found no data when reading from dongle!");
        if let Some(x) = parser.parse(temp[0]) {
            result_vec.push(x);
        }
    }
}
