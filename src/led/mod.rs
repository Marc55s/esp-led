pub mod data;
pub mod engine;

use crate::led::data::LedUpdate;
use crate::led::engine::LedEngine;
use crate::TWDTDriver;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::RmtChannel;
use std::marker::Send;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn spawn_led_thread(
    channel: impl Peripheral<P = impl RmtChannel> + 'static + Send,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static + Send,
    rx: Receiver<LedUpdate>,
    twtd_driver: TWDTDriver<'static>,
    total_leds: usize,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut engine = LedEngine::new(channel, led_pin, total_leds);

        engine.run(rx, twtd_driver);
    })
}
