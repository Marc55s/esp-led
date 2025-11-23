use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::RmtChannel;
use log::*;
use serde::{Deserialize, Serialize};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{SmartLedsWrite, RGB};
use std::thread;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct LedData {
    pub offset: usize,
    pub h: Vec<u8>,
    pub s: Vec<u8>,
    pub v: Vec<u8>,
}

impl LedData {
    pub fn convert_to_iter(self) -> Result<Vec<RGB<u8>>, String> {
        if self.h.len() % 3 != 0 {
            return Err("R length is not divisible by 3".to_string());
        } else if self.s.len() % 3 != 0 {
            return Err("G length is not divisible by 3".to_string());
        } else if self.v.len() % 3 != 0 {
            return Err("B length is not divisible by 3".to_string());
        } else if self.h.len() != self.s.len() || self.h.len() != self.v.len() {
            return Err("RGB lengths are not equal".to_string());
        }

        Ok(self.h.into_iter().zip(self.s).zip(self.v).map(|((h, s), v)| {
            hsv2rgb(Hsv {
                hue: h,
                sat: s,
                val: v,
            })
        }).collect())
    }
}

pub fn led_setup(
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static,
    mut rx: tokio::sync::watch::Receiver<Vec<RGB<u8>>>,
) {
    let mut ws2812 = Ws2812Esp32Rmt::new(channel, led_pin).expect("Failed to create ws2812");
    thread::spawn(move || {
        info!("LED thread started");
        loop {
            if rx.has_changed().expect("has changed boolean") {
                let val = &(*rx.borrow_and_update());
                let _ = ws2812.write(val.iter().cloned());
            }
        }
    });
}
