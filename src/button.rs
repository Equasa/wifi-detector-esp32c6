use defmt::info;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::peripherals::GPIO20;

use crate::BUTTON_PRESS;

#[embassy_executor::task]
pub async fn button(but_pin: GPIO20<'static>) {
    let mut button = Input::new(but_pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        button.wait_for_falling_edge().await;
        info!("Button pressed!");
        BUTTON_PRESS.send(1).await;
        Timer::after(Duration::from_millis(500)).await;
    }
}
