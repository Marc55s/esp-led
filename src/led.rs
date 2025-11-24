use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::{config::TransmitConfig, RmtChannel, TxRmtDriver};
use esp_idf_hal::task::watchdog::TWDTDriver;
use log::{error, info};
use serde::{Deserialize, Serialize};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{SmartLedsWrite, RGB};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct LedData {
    pub offset: usize,
    pub h: Vec<u8>,
    pub s: Vec<u8>,
    pub v: Vec<u8>,
}

pub struct LedUpdate {
    pub offset: usize,
    pub data: Vec<RGB<u8>>,
}

impl LedData {
    pub fn convert_to_iter(&self) -> Result<Vec<RGB<u8>>, String> {
        if self.h.len() != self.s.len() || self.h.len() != self.v.len() {
            return Err("HSV lengths are not equal".to_string());
        }

        Ok(self
            .h
            .clone()
            .into_iter()
            .zip(self.s.clone())
            .zip(self.v.clone())
            .map(|((h, s), v)| {
                hsv2rgb(Hsv {
                    hue: h,
                    sat: s,
                    val: v,
                })
            })
            .collect())
    }
}

impl LedUpdate {
    pub fn from_led_data(data: LedData) -> Result<LedUpdate, String> {
        Ok(LedUpdate {
            offset: data.offset,
            data: data.convert_to_iter()?,
        })
    }
}

pub fn led_setup(
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static,
    rx: Receiver<LedUpdate>,
    mut twtd_driver: TWDTDriver<'static>,
    total_leds: usize,
) {
    // Increase mem_block_num to 4 (or even 8 if you have few channels).
    // This gives the RMT a huge buffer to survive WiFi activity.
    let config = TransmitConfig::new().clock_divider(1).mem_block_num(4);
    let driver = TxRmtDriver::new(channel, led_pin, &config).expect("Failed to create RMT driver");

    // Rmt with custom driver
    let mut ws2812 =
        Ws2812Esp32Rmt::new_with_rmt_driver(driver).expect("Failed to create ws2812 wrapper");

    thread::spawn(move || {
        info!("LED thread started with High-Buffer RMT");
        let mut watchdog = twtd_driver
            .watch_current_task()
            .expect("Failed to create watchdog");

        let mut framebuffer = vec![RGB::new(0, 0, 0); total_leds];

        loop {
            // Keep watchdog alive
            if let Err(e) = watchdog.feed() {
                error!("Failed to feed watchdog: {:?}", e);
            }

            // Non Blocking Channel listening
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(led_update) => {
                    for (i, pixel) in led_update.data.into_iter().enumerate() {
                        let target_index = led_update.offset + i;

                        if target_index < framebuffer.len() {
                            framebuffer[target_index] = pixel;
                        }
                    }

                    if let Err(e) = ws2812.write(framebuffer.iter().cloned()) {
                        error!("Failed to write to LEDs: {:?}", e);
                    }
                }

                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }

                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Channel disconnected, LED thread stopping");
                    break;
                }
            }
        }
    });
}
