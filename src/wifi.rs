use alloc::format;
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
use esp_wifi::wifi::Interfaces;
use esp_wifi::wifi::WifiController;
use esp_wifi::{
    init,
    wifi::WifiDevice,
    EspWifiController,
};

use hashbrown::HashMap;
use heapless::String;

//for display

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
pub async fn wifi(spawner: Spawner, timer: Timer<'static>, wifi: WIFI, r: RNG, radio: RADIO_CLK) {
    let mut rng = Rng::new(r);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timer, rng.clone(), radio).unwrap()
    );

    let (mut controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();

    let wifi_interface = interfaces.sta;

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
    spawner.spawn(sniffer_loop(controller, interfaces));

}


#[embassy_executor::task]
async fn scan_loop(mut controller: WifiController<'static>) {
    let mut map: HashMap<String<32>, u8> = HashMap::with_capacity(HASHMAP_SIZE);
    loop {
        let (results, count) = controller.scan_n(VEC_SIZE).unwrap();
        info!("Found {} networks", count);

        for item in &results {
            map.entry(item.ssid.clone()).or_insert(0);
        }

        Timer2::after(Duration::from_millis(4_000)).await;
    }
}

// #[embassy_executor::task]
// async fn sniffer_loop(mut controller: WifiController<'static>, interfaces:Interfaces<'static>) {
//     let mut sniffer = interfaces.sniffer;
//     sniffer.set_promiscuous_mode(true).unwrap();
//     sniffer.set_receive_cb(|packet| {
//         let _ = match_frames! {
//             packet.data,
//             beacon = BeaconFrame => {
//                 let Some(ssid) = beacon.ssid() else {
//                     return;
//                 };
//                 if critical_section::with(|cs| {
//                     KNOWN_SSIDS.borrow_ref_mut(cs).insert(ssid.to_string())
//                 }) {
//                     println!("Found new AP with SSID: {ssid}");
//                 }
//             }
//         };
//     });
// }

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
