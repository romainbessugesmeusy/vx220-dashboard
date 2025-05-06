use btleplug::platform::{Adapter, Manager};
use btleplug::api::{Manager as _, Central as _, Peripheral as _};
use futures::stream::StreamExt;
use tokio::task;
use std::time::Duration;
use thiserror::Error;

use crate::racebox::protocol::*;
use crate::racebox::parser::{parse_packet, RaceBoxData};

#[derive(Error, Debug)]
pub enum BleError {
    #[error("Failed to create BLE manager: {0}")]
    ManagerCreation(btleplug::Error),
    
    #[error("No BLE adapter found")]
    NoAdapter,
    
    #[error("Failed to start BLE scan: {0}")]
    ScanStart(btleplug::Error),
    
    #[error("Failed to get peripherals: {0}")]
    PeripheralDiscovery(btleplug::Error),
    
    #[error("Failed to connect to device: {0}")]
    Connection(btleplug::Error),
    
    #[error("Failed to discover services: {0}")]
    ServiceDiscovery(btleplug::Error),
    
    #[error("TX characteristic not found")]
    CharacteristicNotFound,
    
    #[error("Failed to subscribe to notifications: {0}")]
    Subscription(btleplug::Error),
    
    #[error("Failed to get notifications: {0}")]
    NotificationSetup(btleplug::Error),
}

impl From<btleplug::Error> for BleError {
    fn from(err: btleplug::Error) -> Self {
        BleError::ManagerCreation(err)
    }
}

pub fn start_ble_listener<F, E>(mut on_data: F, mut on_error: E)
where
    F: FnMut(RaceBoxData) + Send + 'static,
    E: FnMut(BleError) + Send + 'static,
{
    task::spawn(async move {
        let manager = match Manager::new().await {
            Ok(m) => m,
            Err(e) => {
                on_error(BleError::ManagerCreation(e));
                return;
            }
        };
        
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(e) => {
                on_error(BleError::ManagerCreation(e));
                return;
            }
        };
        
        let central = match adapters.into_iter().next() {
            Some(c) => c,
            None => {
                on_error(BleError::NoAdapter);
                return;
            }
        };

        // Scanning strategy:
        // 1. Every 1 second for first 10 seconds
        // 2. Every 3 seconds for next 30 seconds
        // 3. Every 10 seconds indefinitely
        let mut scan_count = 0;
        let mut connected = false;

        while !connected {
            if let Err(e) = central.start_scan(Default::default()).await {
                on_error(BleError::ScanStart(e));
                return;
            }

            // Determine sleep duration based on scan count
            let sleep_duration = if scan_count < 10 {
                Duration::from_secs(1)
            } else if scan_count < 20 {
                Duration::from_secs(3)
            } else {
                Duration::from_secs(10)
            };

            tokio::time::sleep(sleep_duration).await;
            scan_count += 1;

            let peripherals = match central.peripherals().await {
                Ok(p) => p,
                Err(e) => {
                    on_error(BleError::PeripheralDiscovery(e));
                    continue;
                }
            };

            for p in peripherals {
                if let Ok(Some(props)) = p.properties().await {
                    if let Some(local_name) = props.local_name {
                        if local_name.starts_with("RaceBox Micro") {
                            if let Err(e) = p.connect().await {
                                on_error(BleError::Connection(e));
                                continue;
                            }

                            if let Err(e) = p.discover_services().await {
                                on_error(BleError::ServiceDiscovery(e));
                                continue;
                            }

                            let chars = p.characteristics();
                            let tx = match chars.iter().find(|c| c.uuid.to_string() == TX_CHAR_UUID) {
                                Some(c) => c,
                                None => {
                                    on_error(BleError::CharacteristicNotFound);
                                    continue;
                                }
                            };

                            if let Err(e) = p.subscribe(tx).await {
                                on_error(BleError::Subscription(e));
                                continue;
                            }

                            let mut notifications = match p.notifications().await {
                                Ok(n) => n,
                                Err(e) => {
                                    on_error(BleError::NotificationSetup(e));
                                    continue;
                                }
                            };

                            connected = true;
                            while let Some(data) = notifications.next().await {
                                if let Some(parsed) = parse_packet(&data.value) {
                                    on_data(parsed);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
