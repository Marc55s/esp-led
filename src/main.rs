mod wifi;
mod led;

use crate::wifi::wifi_setup;
use crate::led::{led_setup};

use esp_idf_hal::peripherals::Peripherals;
use tokio::sync::watch::channel;
use log::*;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio33;
    let channel_rmt = peripherals.rmt.channel0;
    let modem = peripherals.modem;

    let (tx, rx) = channel(0 as u8);

    info!("check");
    wifi_setup(modem, tx).expect("Wifi setup failed");
    
    led_setup(channel_rmt, led_pin, rx);

    Ok(())
}
