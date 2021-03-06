use clap::{App, Arg};
use env_logger;
use hex::decode;
use minifb::{Key, Window, WindowOptions};
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use plotters_bitmap::BitMapBackend;
use rustymind::{connect_headset, PacketType, Parser, HEADSETID_AUTOCONNECT};
use std::borrow::{Borrow, BorrowMut};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const W: usize = 800;
const H: usize = 1000;
const LABEL: [&str; 2] = ["Attention", "Meditation"];
const EEGLABEL: [&str; 8] = [
    "delta",
    "theta",
    "low-alpha",
    "high-alpha",
    "low-beta",
    "high-beta",
    "low-gamma",
    "mid-gamma",
];

struct BufferWrapper(Vec<u32>);
impl Borrow<[u8]> for BufferWrapper {
    fn borrow(&self) -> &[u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const u8, self.0.len() * 4) }
    }
}
impl BorrowMut<[u8]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u8, self.0.len() * 4) }
    }
}
impl Borrow<[u32]> for BufferWrapper {
    fn borrow(&self) -> &[u32] {
        self.0.as_slice()
    }
}
impl BorrowMut<[u32]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u32] {
        self.0.as_mut_slice()
    }
}

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
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    let headset = matches
        .value_of("HEADSET_ID")
        .map_or(HEADSETID_AUTOCONNECT.to_vec(), |v| {
            decode(v).expect("Hex decoding failed")
        });
    let path = matches.value_of("dongle-path").unwrap();
    let mut port = connect_headset(path, &headset[..])?;
    let mut read_buf: Vec<u8> = vec![0; 2048];
    let mut parser = Parser::new();
    let mut esense = vec![VecDeque::new(); 2];
    let mut eeg = vec![VecDeque::new(); 8];
    let mut draw_buf = BufferWrapper(vec![0u32; W * H]);
    let mut window = Window::new("mindwave plot", W, H, WindowOptions::default())?;
    let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
        draw_buf.borrow_mut(),
        (W as u32, H as u32),
    )?
    .into_drawing_area();
    root.fill(&BLACK)?;
    let (upper, lower) = root.split_vertically(400);
    let mut chart_up = ChartBuilder::on(&upper)
        .margin(10)
        .caption(
            "Real-time eSense plot",
            ("sans-serif", 15).into_font().color(&GREEN),
        )
        .set_all_label_area_size(40)
        .build_cartesian_2d(0..110, 0..110)?;
    let mut chart_low = ChartBuilder::on(&lower)
        .margin(10)
        .caption(
            "Real-time brainwaves plot",
            ("sans-serif", 15).into_font().color(&GREEN),
        )
        .set_all_label_area_size(40)
        .build_cartesian_2d(0..110, 0.0..300.0)?;
    chart_up
        .configure_mesh()
        .disable_mesh()
        .label_style(("sans-serif", 15).into_font().color(&GREEN))
        .x_labels(1)
        .y_labels(10)
        .axis_style(&GREEN)
        .draw()?;
    chart_low
        .configure_mesh()
        .disable_mesh()
        .label_style(("sans-serif", 15).into_font().color(&GREEN))
        .x_labels(1)
        .y_labels(8)
        .axis_style(&GREEN)
        .draw()?;
    let cs_up = chart_up.into_chart_state();
    let cs_low = chart_low.into_chart_state();
    drop(root);
    drop(upper);
    drop(lower);

    while window.is_open() && !window.is_key_down(Key::Escape) && running.load(Ordering::SeqCst) {
        let bytes_read = port.read(read_buf.as_mut_slice()).expect(
            "Found no data when reading from dongle. Please make sure headset is connected.",
        );
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            draw_buf.borrow_mut(),
            (W as u32, H as u32),
        )?
        .into_drawing_area();
        let (upper, lower) = root.split_vertically(400);
        let mut chart_up = cs_up.clone().restore(&upper);
        let mut chart_low = cs_low.clone().restore(&lower);
        chart_up.plotting_area().fill(&BLACK)?;
        chart_up
            .configure_mesh()
            .bold_line_style(&GREEN.mix(0.2))
            .light_line_style(&TRANSPARENT)
            .draw()?;
        chart_low.plotting_area().fill(&BLACK)?;
        chart_low
            .configure_mesh()
            .bold_line_style(&GREEN.mix(0.2))
            .light_line_style(&TRANSPARENT)
            .draw()?;
        for i in 0..bytes_read {
            if let Some(x) = parser.parse(read_buf[i]) {
                for r in x {
                    match r {
                        PacketType::Attention(value) => {
                            esense[0].push_back(value as i32);
                        }
                        PacketType::Meditation(value) => {
                            esense[1].push_back(value as i32);
                        }
                        PacketType::AsicEeg(value) => {
                            eeg[0].push_back((value.delta / 10_000) as f64);
                            eeg[1].push_back((value.theta / 10_000) as f64);
                            eeg[2].push_back((value.low_alpha / 10_000) as f64);
                            eeg[3].push_back((value.high_alpha / 10_000) as f64);
                            eeg[4].push_back((value.low_beta / 10_000) as f64);
                            eeg[5].push_back((value.high_beta / 10_000) as f64);
                            eeg[6].push_back((value.low_gamma / 10_000) as f64);
                            eeg[7].push_back((value.mid_gamma / 10_000) as f64);
                        }
                        _ => (),
                    }
                }
            }
        }
        if esense[0].len() == 100 {
            esense[0].pop_front();
            esense[1].pop_front();
        }
        if eeg[0].len() == 100 {
            for n in 0..8 {
                eeg[n].pop_front();
            }
        }
        for (idx, esense) in (0..).zip(esense.iter()) {
            chart_up
                .draw_series(LineSeries::new(
                    (1..).zip(esense.iter()).map(|(a, b)| (a, *b)),
                    &Palette99::pick(idx),
                ))?
                .label(LABEL[idx])
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
                });
        }
        chart_up
            .configure_series_labels()
            .legend_area_size(10)
            .position(SeriesLabelPosition::UpperRight)
            .background_style(&WHITE.mix(0.5))
            .border_style(&BLACK)
            .draw()?;
        for (idx, eeg) in (0..).zip(eeg.iter()) {
            chart_low
                .draw_series(LineSeries::new(
                    (1..).zip(eeg.iter()).map(|(a, b)| (a, *b)),
                    &Palette99::pick(idx),
                ))?
                .label(EEGLABEL[idx])
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
                });
        }
        chart_low
            .configure_series_labels()
            .legend_area_size(5)
            .position(SeriesLabelPosition::UpperRight)
            .background_style(&WHITE.mix(0.5))
            .border_style(&BLACK)
            .draw()?;
        drop(root);
        drop(chart_up);
        drop(chart_low);
        drop(upper);
        drop(lower);
        window.set_title("Mindwave real-time plot");
        window.update_with_buffer(draw_buf.borrow(), W, H)?;
    }
    Ok(())
}
