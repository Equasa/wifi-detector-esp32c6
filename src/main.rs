#![no_std]
#![no_main]

use alloc::string::String;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use panic_rtt_target as _;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

extern crate alloc;

mod button;
mod controller;
mod display;
mod wifi;

static DISPLAY_VALUE: Channel<CriticalSectionRawMutex, (String, u8), 1> = Channel::new();

static BUTTON_PRESS: Channel<CriticalSectionRawMutex, u8, 1> = Channel::new();

static PKT_SENDER: Channel<CriticalSectionRawMutex, ([u8; 6], [u8; 6]), 10> = Channel::new();

static SSID_MAC: Channel<CriticalSectionRawMutex, (String, [u8; 6]), 1> = Channel::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(size: 100 * 1024);

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    let timer1 = TimerGroup::new(peripherals.TIMG0);

    esp_hal_embassy::init(timer0.alarm0);

    spawner
        .spawn(display::display(
            peripherals.GPIO21,
            peripherals.GPIO22,
            peripherals.I2C0,
        ))
        .unwrap();

    spawner
        .spawn(wifi::wifi(
            spawner,
            timer1.timer0,
            peripherals.WIFI,
            peripherals.RNG,
            peripherals.RADIO_CLK,
        ))
        .unwrap();

    let button_pin = peripherals.GPIO20;

    spawner.spawn(button::button(button_pin)).unwrap();

    spawner.spawn(controller::handle_addresses()).unwrap();

    spawner.spawn(controller::handle_name()).unwrap();
}
