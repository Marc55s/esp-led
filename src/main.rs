mod wifi;
mod led;

use crate::wifi::wifi_setup;
use crate::led::led_setup;

use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();


    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio33; // GPIO 33
    let channel = peripherals.rmt.channel0;
    let modem = peripherals.modem;

    wifi_setup(modem).expect("Wifi setup failed");

    led_setup(channel, led_pin);

    Ok(())
}
