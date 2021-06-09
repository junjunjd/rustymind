use env_logger;
use minifb::{Key, Window, WindowOptions};
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use plotters_bitmap::BitMapBackend;
use rustymind::connect_headset;
use rustymind::PacketType;
use rustymind::Parser;
use std::borrow::{Borrow, BorrowMut};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const W: usize = 800;
const H: usize = 1000;
const LABEL: [&str; 2] = ["Attention", "Meditation"];
const EGGLABEL: [&str; 8] = [
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
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    //let headset = [0xa2, 0x6c];
    let headset = [0xc2];
    let path = "/dev/tty.usbserial-14140";
    let mut port = connect_headset(path, &headset)?;
    let mut temp: Vec<u8> = vec![0; 2048];
    let mut parser = Parser::new();
    let mut data = vec![VecDeque::new(); 2];
    let mut egg = vec![VecDeque::new(); 8];
    let mut buf = BufferWrapper(vec![0u32; W * H]);

    let mut window = Window::new("mindwave plot", W, H, WindowOptions::default())?;
    let root =
        BitMapBackend::<BGRXPixel>::with_buffer_and_format(buf.borrow_mut(), (W as u32, H as u32))?
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

    chart_up
        .configure_mesh()
        .disable_mesh()
        .label_style(("sans-serif", 15).into_font().color(&GREEN))
        .x_labels(1)
        .y_labels(10)
        .y_desc("eSense")
        .axis_style(&GREEN)
        .draw()?;

    let mut chart_low = ChartBuilder::on(&lower)
        .margin(10)
        .caption(
            "Real-time brainwaves plot",
            ("sans-serif", 15).into_font().color(&GREEN),
        )
        .set_all_label_area_size(40)
        .build_cartesian_2d(0..110, 0.0..300.0)?;

    chart_low
        .configure_mesh()
        .disable_mesh()
        .label_style(("sans-serif", 15).into_font().color(&GREEN))
        .x_labels(1)
        .y_labels(8)
        .y_desc("EGG power")
        .axis_style(&GREEN)
        .draw()?;

    let cs_up = chart_up.into_chart_state();
    let cs_low = chart_low.into_chart_state();
    drop(root);
    drop(upper);
    drop(lower);
    while window.is_open() && !window.is_key_down(Key::Escape) && running.load(Ordering::SeqCst) {
        let byte_buf = port.read(temp.as_mut_slice()).expect(
            "Found no data when reading from connect_headset! Please make sure headset is connected.",
        );
        for i in 0..byte_buf {
            if let Some(x) = parser.parse(temp[i]) {
                for r in x {
                    match r {
                        PacketType::Attention(value) => {
                            data[0].push_back(value as i32);
                        }
                        PacketType::Meditation(value) => {
                            data[1].push_back(value as i32);
                        }
                        PacketType::AsicEgg(value) => {
                            egg[0].push_back((value.delta / 10_000) as f64);
                            egg[1].push_back((value.theta / 10_000) as f64);
                            egg[2].push_back((value.low_alpha / 10_000) as f64);
                            egg[3].push_back((value.high_alpha / 10_000) as f64);
                            egg[4].push_back((value.low_beta / 10_000) as f64);
                            egg[5].push_back((value.high_beta / 10_000) as f64);
                            egg[6].push_back((value.low_gamma / 10_000) as f64);
                            egg[7].push_back((value.mid_gamma / 10_000) as f64);
                        }
                        _ => (),
                    }
                }
            }
        }

        if data[0].len() == 100 {
            data[0].pop_front();
            data[1].pop_front();
        }
        if egg[0].len() == 100 {
            for n in 0..8 {
                egg[n].pop_front();
            }
        }
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            buf.borrow_mut(),
            (W as u32, H as u32),
        )?
        .into_drawing_area();
        let (upper, lower) = root.split_vertically(400);
        let mut chart_up = cs_up.clone().restore(&upper);
        chart_up.plotting_area().fill(&BLACK)?;

        chart_up
            .configure_mesh()
            .bold_line_style(&GREEN.mix(0.2))
            .light_line_style(&TRANSPARENT)
            .draw()?;

        let mut chart_low = cs_low.clone().restore(&lower);
        chart_low.plotting_area().fill(&BLACK)?;

        chart_low
            .configure_mesh()
            .bold_line_style(&GREEN.mix(0.2))
            .light_line_style(&TRANSPARENT)
            .draw()?;

        for (idx, data) in (0..).zip(data.iter()) {
            chart_up
                .draw_series(LineSeries::new(
                    (1..).zip(data.iter()).map(|(a, b)| (a, *b)),
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

        for (idx, egg) in (0..).zip(egg.iter()) {
            chart_low
                .draw_series(LineSeries::new(
                    (1..).zip(egg.iter()).map(|(a, b)| (a, *b)),
                    &Palette99::pick(idx),
                ))?
                .label(EGGLABEL[idx])
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
        window.update_with_buffer(buf.borrow(), W, H)?;
    }
    Ok(())
}
