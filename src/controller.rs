use alloc::string::String;

use alloc::string::ToString;
use alloc::vec::Vec;
use defmt::info;

use hashbrown::HashMap;

use crate::DISPLAY_VALUE;
use crate::PKT_SENDER;
use crate::SSID_MAC;

const HASHMAP_SIZE: usize = 200;

#[embassy_executor::task]
pub async fn handle_addresses() {
    let mut num_connections: HashMap<[u8; 6], u8> = HashMap::with_capacity(HASHMAP_SIZE);
    let mut connections: HashMap<[u8; 6], [u8; 6]> = HashMap::with_capacity(HASHMAP_SIZE);

    loop {
        let (src, dst) = PKT_SENDER.receive().await;

        if let Some(old_dst) = connections.get(&src).cloned() {
            if let Some(cnt) = num_connections.get_mut(&old_dst) {
                *cnt = cnt.saturating_sub(1);
            }
        }
        connections.insert(src.clone(), dst.clone());

        let count = num_connections.entry(dst.clone()).or_insert(0);
        *count += 1;
        info!(
            "count = {}",
            *num_connections
                .entry([140, 89, 115, 248, 225, 24])
                .or_default()
        );
        let _ = DISPLAY_VALUE.try_send((
            String::from("Lord Voldemodem"),
            *num_connections
                .entry([140, 89, 115, 248, 225, 24])
                .or_default(),
        ));
    }
}

#[embassy_executor::task]
pub async fn handle_name() {
    let mut bssid_to_ssid: HashMap<[u8; 6], String> = HashMap::with_capacity(HASHMAP_SIZE);

    loop {
        let (ssid, bssid) = SSID_MAC.receive().await;
        bssid_to_ssid.entry(bssid.clone()).or_insert(ssid.clone());
        info!(
            "BSSID: {}, SSID: {} ",
            parse_bssid(&bssid).as_str(),
            ssid.as_str()
        )
    }
}

fn parse_bssid(data: &[u8]) -> String {
    let address1: Vec<String> = data.iter().map(|&x| x.to_string()).collect();
    return address1.join(".");
}
