mod http;
mod led;
mod wifi;

use crate::http::http_routes;
use crate::wifi::wifi_setup;

use crate::led::data::LedUpdate;
use crate::led::spawn_led_thread;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::watchdog::TWDTDriver;
use log::*;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

const LEDS: usize = 100;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio33;
    let channel_rmt = peripherals.rmt.channel0;
    let modem = peripherals.modem;

    let (tx, rx) = sync_channel::<LedUpdate>(5);

    info!("check");
    let _wifi = wifi_setup(modem).expect("Wifi setup failed");
    http_routes(tx).expect("HTTP routes failed");

    // Watchdog
    let twdt_config = esp_idf_hal::task::watchdog::TWDTConfig {
        duration: Duration::from_secs(3),
        panic_on_trigger: true,
        ..Default::default()
    };

    // Create the driver (this starts the hardware timer)
    let twdt_driver = TWDTDriver::new(peripherals.twdt, &twdt_config)?;
    let _led_thread = spawn_led_thread(channel_rmt, led_pin, rx, twdt_driver, LEDS);

    loop {
        info!("Still alive...");
        std::thread::sleep(core::time::Duration::from_secs(60));
    }
}
