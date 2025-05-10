
use alloc::string::ParseError;
use alloc::string::String;
use alloc::vec::Vec;
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_time::Duration;
use embassy_time::Timer as Timer2;
use esp_hal::{
    peripherals::{RADIO_CLK, RNG, WIFI},
    rng::Rng,
    timer::timg::Timer,
};
use esp_wifi::wifi::Sniffer;
use esp_wifi::wifi::WifiController;
use esp_wifi::{init, wifi::WifiDevice, EspWifiController};

use alloc::string::ToString;


use hashbrown::HashMap;


use crate::DISPLAY_VALUE;

const VEC_SIZE: usize = 16;
const HASHMAP_SIZE: usize = 128;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[embassy_executor::task]
pub async fn wifi(
    spawner: Spawner,
    timer: Timer<'static>,
    wifi: WIFI<'static>,
    r: RNG<'static>,
    radio: RADIO_CLK<'static>,
) {
    let mut rng = Rng::new(r);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timer, rng.clone(), radio).unwrap()
    );

    let (mut controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();

    let wifi_interface = interfaces.sta;
    let sniffer = interfaces.sniffer;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (_stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );
    controller
        .set_mode(esp_wifi::wifi::WifiMode::ApSta)
        .unwrap();

    spawner.spawn(net_task(runner)).ok();

    controller
        .start_async()
        .await
        .expect("failed to start WiFi driver");

    info!("got here");

    spawner.spawn(scan_loop(controller)).unwrap();
    spawner.spawn(sniffer_loop(sniffer)).unwrap();
}

#[embassy_executor::task]
async fn scan_loop(mut controller: WifiController<'static>) {
    let mut map: HashMap<String, u8> = HashMap::with_capacity(HASHMAP_SIZE);
    loop {
        let results = controller.scan_n(VEC_SIZE).unwrap();
        info!("Found {} networks", results.len());

        for item in &results {
            map.entry(item.ssid.clone()).or_insert(0);
        }

        Timer2::after(Duration::from_millis(4_000)).await;
    }
}

fn parse_wifi_packet(data: &[u8]) -> Result<(String, String), ParseError> {
    let address1: Vec<String> = data[4..8].iter().map(|&x| x.to_string()).collect();
    let address2: Vec<String> = data[9..13].iter().map(|&x| x.to_string()).collect();


    return Ok((address1.join("."), address2.join(".")));
}

#[embassy_executor::task]
async fn sniffer_loop(mut sniffer: Sniffer) {
    sniffer.set_promiscuous_mode(true).unwrap();
    sniffer.set_receive_cb(|packet| {
        let data = packet.data;
        let (adr1, adr2) = parse_wifi_packet(data).unwrap();
        info!("MAC: {} → {} )", adr1.as_str(), adr2.as_str());
    });
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
