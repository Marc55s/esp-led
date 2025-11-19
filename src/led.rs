use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::RmtChannel;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::SmartLedsWrite;
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;
use std::thread;
use log::*;

use crate::wifi::HsvColor;

pub fn led_setup(
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static,
    mut rx: tokio::sync::watch::Receiver<Vec<HsvColor>>,
) {
    let mut ws2812 = Ws2812Esp32Rmt::new(channel, led_pin).unwrap();
    thread::spawn(move || {
        info!("LED thread started");
        loop {
            let val = &(*rx.borrow_and_update());
            let led_from_channel = val.iter().map(|e| e.to_smart_hsv()).map(|e| hsv2rgb(e));
            let _ = ws2812.write(led_from_channel);
            sleep(Duration::from_millis(100));
        }
    });
}
