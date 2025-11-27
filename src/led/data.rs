use serde::{Deserialize, Serialize};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{RGB};
use std::ops::{Deref, DerefMut};

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

pub struct LedBuffer(Vec<RGB<u8>>);

impl Deref for LedBuffer {
    type Target = Vec<RGB<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LedBuffer {
    pub fn new(length: usize) -> Self {
        Self(vec![RGB::new(0, 0, 0); length])
    }
}

impl LedData {
        pub fn to_rgb_vec(&self) -> Result<Vec<RGB<u8>>, String> {
        if self.h.len() != self.s.len() || self.h.len() != self.v.len() {
            return Err("HSV lengths are not equal".to_string());
        }

        Ok(self
            .h
            .iter()
            .zip(self.s.iter())
            .zip(self.v.iter())
            .map(|((&h, &s), &v)| {
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
            data: data.to_rgb_vec()?,
        })
    }
}

