mod wifi;
mod led;
mod http;

use crate::wifi::{wifi_setup};
use crate::led::{led_setup};
use crate::http::{http_routes};

use esp_idf_hal::peripherals::Peripherals;
use smart_leds::RGB;
use std::sync::mpsc::channel;
use log::*;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio33;
    let channel_rmt = peripherals.rmt.channel0;
    let modem = peripherals.modem;

    let (tx, rx) = channel::<Vec<RGB<u8>>>();

    info!("check");
    wifi_setup(modem).expect("Wifi setup failed");
    http_routes(tx).expect("HTTP routes failed");

    
    led_setup(channel_rmt, led_pin, rx);

    Ok(())
}
