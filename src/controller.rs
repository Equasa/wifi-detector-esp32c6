use alloc::string::String;

use alloc::string::ToString;
use alloc::vec::Vec;
// use critical_section::Mutex;
use defmt::info;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;

use embassy_time::{Duration, Timer};


use hashbrown::HashMap;

use embassy_sync::lazy_lock::LazyLock;

use core::cell::RefCell;

use crate::DISPLAY_VALUE;
use crate::PKT_SENDER;
use crate::SSID_MAC;

const HASHMAP_SIZE: usize = 200;

static NUM_CONNECTIONS: LazyLock<
    BlockingMutex<CriticalSectionRawMutex, RefCell<HashMap<[u8; 6], u8>>>,
> = LazyLock::new(|| BlockingMutex::new(RefCell::new(HashMap::with_capacity(HASHMAP_SIZE))));

static CONNECTIONS: LazyLock<
    BlockingMutex<CriticalSectionRawMutex, RefCell<HashMap<[u8; 6], [u8; 6]>>>,
> = LazyLock::new(|| BlockingMutex::new(RefCell::new(HashMap::with_capacity(HASHMAP_SIZE))));

static BSSID_TO_SSID: LazyLock<
    BlockingMutex<CriticalSectionRawMutex, RefCell<HashMap<[u8; 6], String>>>,
> = LazyLock::new(|| BlockingMutex::new(RefCell::new(HashMap::with_capacity(HASHMAP_SIZE))));

#[embassy_executor::task]
pub async fn handle_addresses() {
    let num_lock = NUM_CONNECTIONS.get();
    let conn_lock = CONNECTIONS.get();

    loop {
        let (src, dst) = PKT_SENDER.receive().await;

        num_lock.lock(|num_cell| {
            let mut num_connections = num_cell.borrow_mut();

            conn_lock.lock(|connections_cell| {
                let mut connections = connections_cell.borrow_mut();

                if let Some(old_dst) = connections.get(&src).cloned() {
                    if let Some(cnt) = num_connections.get_mut(&old_dst) {
                        *cnt = cnt.saturating_sub(1);
                    }
                }
                connections.insert(src.clone(), dst.clone());

                let count = num_connections.entry(dst.clone()).or_insert(0);
                *count += 1;
            });
        });
    }
}

#[embassy_executor::task]
pub async fn handle_name() {
    let bssid_to_ssid_lock = BSSID_TO_SSID.get();

    loop {
        let (ssid, bssid) = SSID_MAC.receive().await;
        bssid_to_ssid_lock.lock(|bssid_to_ssid_cell| {
            let mut bssid_to_ssid = bssid_to_ssid_cell.borrow_mut();

            bssid_to_ssid.entry(bssid.clone()).or_insert(ssid.clone());
            // info!(
            //     "BSSID: {}, SSID: {} ",
            //     parse_bssid(&bssid).as_str(),
            //     ssid.as_str()
            // )
        });
        Timer::after(Duration::from_millis(200)).await;

    }
}

#[embassy_executor::task]
pub async fn ssid_count_pairer() {
    let num_lock = NUM_CONNECTIONS.get();
    let bssid_to_ssid_lock = BSSID_TO_SSID.get();

    loop {
        let mut snapshot = Vec::new();
        num_lock.lock(|num_cell| {
            bssid_to_ssid_lock.lock(|bssid_to_ssid_lock_cell| {
                let bssid_to_ssid = bssid_to_ssid_lock_cell.borrow();
                for (&bssid, &count) in num_cell.borrow().iter() {

                    let ssid = bssid_to_ssid.get(&bssid);
                    if ssid != None {
                        snapshot.push((bssid_to_ssid.get(&bssid).cloned(), count));
                    }
                }

            });

            for (ssid, count) in snapshot {
                //info!("{}, {}", ssid.unwrap().as_str(),count);
                let _ = DISPLAY_VALUE.try_send((ssid.clone().unwrap().to_string(), count));
            }

        });

        Timer::after(Duration::from_millis(200)).await;
    }
}


fn parse_bssid(data: &[u8]) -> String {
    let address1: Vec<String> = data.iter().map(|&x| x.to_string()).collect();
    return address1.join(".");
}
