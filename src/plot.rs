use minifb::{Key, KeyRepeat, Window, WindowOptions};
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use plotters_bitmap::BitMapBackend;
use rand::Rng;
use rustymind::dongle;
use rustymind::PacketType;
use rustymind::Parser;
use std::borrow::{Borrow, BorrowMut};
use std::collections::VecDeque;
use std::error::Error;
use std::time::SystemTime;

const W: usize = 480;
const H: usize = 320;
const labels: [&str; 2] = ["Attention", "Meditation"];
const SAMPLE_RATE: i32 = 10_000;
const FREAME_RATE: i32 = 30;

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

fn get_window_title(fx: f64, fy: f64, iphase: f64) -> String {
    format!(
        "x={:.1}Hz, y={:.1}Hz, phase={:.1} +/-=Adjust y 9/0=Adjust x <Esc>=Exit",
        fx, fy, iphase
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut port = dongle();
    let mut temp: Vec<u8> = vec![0; 2048];
    let mut parser = Parser::new();

    // from pitson
    let mut data = vec![VecDeque::new(), VecDeque::new()];

    let mut rng = rand::thread_rng();

    let mut buf = BufferWrapper(vec![0u32; W * H]);

    let mut fx: f64 = 1.0;
    let mut fy: f64 = 1.1;
    let mut xphase: f64 = 0.0;
    let mut yphase: f64 = 0.1;

    let mut window = Window::new("mindwave plot", W, H, WindowOptions::default())?;
    let root =
        BitMapBackend::<BGRXPixel>::with_buffer_and_format(buf.borrow_mut(), (W as u32, H as u32))?
            .into_drawing_area();
    root.fill(&BLACK)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .set_all_label_area_size(30)
        .build_cartesian_2d(0..110, 0..110)?;

    chart
        .configure_mesh()
        .label_style(("sans-serif", 15).into_font().color(&GREEN))
        .axis_style(&GREEN)
        .draw()?;

    let cs = chart.into_chart_state();
    drop(root);

    //let mut data = VecDeque::new();
    let start_ts = SystemTime::now();
    let mut last_flushed = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        loop {
            let mut bytes_read = port
                .read(temp.as_mut_slice())
                .expect("Found no data when reading from dongle!");
            for i in 0..bytes_read {
                if let Some(x) = parser.parse(temp[i]) {
                    for r in x {
                        match r {
                            PacketType::Attention(value) => {
                                data[0].push_back(value as i32);
                                println!("got attention = {:#?}", value);
                            }
                            PacketType::Meditation(value) => {
                                data[1].push_back(value as i32);
                                println!("got meditation = {:#?}", value);
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
            let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
                buf.borrow_mut(),
                (W as u32, H as u32),
            )?
            .into_drawing_area();
            let mut chart = cs.clone().restore(&root);
            chart.plotting_area().fill(&BLACK)?;

            chart
                .configure_mesh()
                .bold_line_style(&GREEN.mix(0.2))
                .light_line_style(&TRANSPARENT)
                .draw()?;

            for (idx, data) in (0..).zip(data.iter()) {
                chart
                    .draw_series(LineSeries::new(
                        (1..).zip(data.iter()).map(|(a, b)| (a, *b)),
                        &Palette99::pick(idx),
                    ))?
                    .label(labels[idx])
                    .legend(move |(x, y)| {
                        Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
                    });
            }
            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()?;

            drop(root);
            drop(chart);
            window.update_with_buffer(buf.borrow(), W, H)?;
        }
    }

    Ok(())
}
