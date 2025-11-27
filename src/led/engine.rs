use crate::led::data::LedUpdate;
use crate::led::data::LedBuffer;
use anyhow::Result;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::{config::TransmitConfig, RmtChannel, TxRmtDriver};
use esp_idf_hal::task::watchdog::TWDTDriver;
use log::{error, info};
use smart_leds::SmartLedsWrite;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

type LEDDriver<'a> = ws2812_esp32_rmt_driver::Ws2812Esp32Rmt<'a>;

pub struct LedEngine<'a> {
    pub buf: LedBuffer,
    pub led_driver: LEDDriver<'a>,
    pub total_leds: usize,
}

impl LedEngine<'_> {
    pub fn new(
        channel: impl Peripheral<P = impl RmtChannel> + 'static,
        led_pin: impl Peripheral<P = impl OutputPin> + 'static,
        led_count: usize,
    ) -> Self {
        if let Ok(ws2812) = create_led_driver(channel, led_pin) {
            Self {
                buf: LedBuffer::new(),
                led_driver: ws2812,
                total_leds: led_count,
            }
        } else {
            panic!("Failed to create LED driver");
        }
    }

    pub fn run(&mut self, rx: Receiver<LedUpdate>, mut twtd_driver: TWDTDriver<'static>) {
        info!("LED thread started with High-Buffer RMT");

        let mut watchdog = twtd_driver
            .watch_current_task()
            .expect("Failed to create watchdog");

        let mut feed_wd = move || {
            if let Err(e) = watchdog.feed() {
                error!("Failed to feed watchdog: {:?}", e);
            }
        };

        loop {
            feed_wd();
            // Non Blocking Channel listening
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(led_update) => {
                    self.update(led_update);
                    self.flush();
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
    }

    pub fn update(&mut self, update: LedUpdate) {
        for (i, pixel) in update.data.into_iter().enumerate() {
            let target_index = update.offset + i;

            if target_index < self.total_leds {
                self.buf[target_index] = pixel;
            }
        }
    }

    pub fn flush(&mut self) {
        if let Err(e) = self.led_driver.write(self.buf.iter().cloned()) {
            error!("Failed to write to LEDs: {:?}", e);
        }
    }
}

fn create_led_driver<'a>(
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
    led_pin: impl Peripheral<P = impl OutputPin> + 'static,
) -> Result<LEDDriver<'a>> {
    // Increase mem_block_num to 4 (or even 8 if you have few channels).
    // This gives the RMT a huge buffer to survive WiFi activity.
    let config = TransmitConfig::new().clock_divider(1).mem_block_num(4);
    let driver = TxRmtDriver::new(channel, led_pin, &config).expect("Failed to create RMT driver");

    match Ws2812Esp32Rmt::new_with_rmt_driver(driver) {
        Ok(led_driver) => Ok(led_driver),
        Err(e) => Err(anyhow::Error::from(e)),
    }
}
