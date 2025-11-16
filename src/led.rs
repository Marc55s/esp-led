use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::RmtChannel;
use esp_idf_sys::esp_random;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::SmartLedsWrite;
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

pub fn led_setup(
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static
) {
    
    let mut ws2812 = Ws2812Esp32Rmt::new(channel, led_pin).unwrap();

    println!("Start NeoPixel rainbow!");

    let mut hue = unsafe { esp_random() } as u8;

    loop {
        let pixels = std::iter::repeat(hsv2rgb(Hsv {
            hue: 200,
            sat: 255,
            val: 8,
        }))
        .take(300);
        ws2812.write(pixels).unwrap();

        sleep(Duration::from_millis(500));

        // hue = hue.wrapping_add(5);
    }
}
