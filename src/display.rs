use alloc::string::ToString;
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_6X13, FONT_7X13};
use embedded_graphics::mono_font::iso_8859_1::FONT_10X20;
use esp_hal::peripherals::I2C0;
use esp_hal::peripherals::{GPIO21, GPIO22};
use esp_hal::time::Rate;
use panic_rtt_target as _;

use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::i2c::master::{Config, I2c};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::{BUTTON_PRESS, DISPLAY_VALUE, MAC_ADRESSES};

#[embassy_executor::task]
pub async fn display(sda: GPIO21<'static>, scl: GPIO22<'static>, i2c: I2C0<'static>) {
    let i2c = I2c::new(i2c, Config::default().with_frequency(Rate::from_khz(400)))
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    let WAITING_MESSAGE = MonoTextStyleBuilder::new()
        .font(&FONT_6X13)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline(
        "No networks found yet",
        Point::new(0, 0),
        WAITING_MESSAGE,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    display.flush().unwrap();

    loop {
        BUTTON_PRESS.receive().await;
        let (name, count) = DISPLAY_VALUE.receive().await;
        //let (s, t) = MAC_ADRESSES.receive().await;

        display.clear(BinaryColor::Off).unwrap();
        Text::with_baseline(&name, Point::new(0, 0), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&count.to_string(), Point::new(0, 10), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
    }
}
