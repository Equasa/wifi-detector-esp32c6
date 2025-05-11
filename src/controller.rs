use alloc::string::String;

use alloc::string::ToString;
use alloc::vec::Vec;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::lazy_lock::LazyLock;

use embassy_time::{Duration, Timer};

use hashbrown::HashMap;

use core::cell::RefCell;

use crate::BUTTON_PRESS;
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

static NETWORK_LIST: LazyLock<
    BlockingMutex<CriticalSectionRawMutex, RefCell<Vec<(String, u8)>>>,
> = LazyLock::new(|| BlockingMutex::new(RefCell::new(Vec::new())));

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
        });
        Timer::after(Duration::from_millis(200)).await;
    }
}

#[embassy_executor::task]
pub async fn ssid_count_pairer() {
    let num_lock = NUM_CONNECTIONS.get();
    let ssid_lock = BSSID_TO_SSID.get();
    let list_lock = NETWORK_LIST.get();

    loop {
        let mut updates: Vec<(String, u8)> = Vec::new();
        num_lock.lock(|num_cell| {
            ssid_lock.lock(|ssid_cell| {
                let bssid_map = ssid_cell.borrow();
                for (&bssid, &count) in num_cell.borrow().iter() {
                    if let Some(ssid) = bssid_map.get(&bssid) {
                        updates.push((ssid.clone(), count));
                    }
                }
            });
        });
        list_lock.lock(|list_cell| {
            let list = &mut *list_cell.borrow_mut();
            for (ssid, count) in updates.drain(..) {
                match list.iter_mut().find(|(s, _)| s == &ssid) {
                    Some((_, existing_count)) => *existing_count = count,
                    None => list.push((ssid, count)),
                }
            }
        });

        Timer::after(Duration::from_millis(200)).await;
    }
}

#[embassy_executor::task]
pub async fn browse_networks() {
    let list_lock = NETWORK_LIST.get();
    let mut idx: usize = 0;

    loop {
        BUTTON_PRESS.receive().await;

        let entry = list_lock.lock(|list_cell| {
            let list = list_cell.borrow();
            if list.is_empty() {
                None
            } else {
                if idx >= list.len() {
                    idx = 0;
                }
                let e = list[idx].clone();
                idx += 1;
                Some(e)
            }
        });

        if let Some((ssid, count)) = entry {
            DISPLAY_VALUE.send((ssid, count)).await;
        }
    }
}

fn parse_bssid(data: &[u8]) -> String {
    let address1: Vec<String> = data.iter().map(|&x| x.to_string()).collect();
    return address1.join(".");
}
