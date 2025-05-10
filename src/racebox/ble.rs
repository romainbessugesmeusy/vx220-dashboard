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
            Ok(m) => {
                crate::racebox_log!(log::Level::Info, "BLE manager created successfully");
                m
            },
            Err(e) => {
                crate::racebox_log!(log::Level::Error, "Failed to create BLE manager: {e}");
                on_error(BleError::ManagerCreation(e));
                return;
            }
        };
        
        let adapters = match manager.adapters().await {
            Ok(a) => {
                crate::racebox_log!(log::Level::Info, "BLE adapters found: {}", a.len());
                a
            },
            Err(e) => {
                crate::racebox_log!(log::Level::Error, "Failed to get BLE adapters: {e}");
                on_error(BleError::ManagerCreation(e));
                return;
            }
        };
        
        let central = match adapters.into_iter().next() {
            Some(c) => {
                crate::racebox_log!(log::Level::Info, "Using BLE adapter");
                c
            },
            None => {
                crate::racebox_log!(log::Level::Error, "No BLE adapter found");
                on_error(BleError::NoAdapter);
                return;
            }
        };

        let scan_result = central.start_scan(Default::default()).await;
        if let Err(e) = scan_result {
            let err_str = format!("{e}");
            if err_str.contains("org.bluez.Error.InProgress") || err_str.contains("Operation already in progress") {
                crate::racebox_log!(log::Level::Warn, "Scan already in progress, continuing");
                // Ignore and continue
            } else {
                crate::racebox_log!(log::Level::Error, "Failed to start BLE scan: {e}");
                on_error(BleError::ScanStart(e));
                return;
            }
        } else {
            crate::racebox_log!(log::Level::Info, "BLE scan started");
        }

        // Scanning strategy:
        // 1. Every 1 second for first 10 seconds
        // 2. Every 3 seconds for next 30 seconds
        // 3. Every 10 seconds indefinitely
        let mut scan_count = 0;
        let mut connected = false;

        while !connected {
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
                Ok(p) => {
                    crate::racebox_log!(log::Level::Debug, "Found {} peripherals", p.len());
                    p
                },
                Err(e) => {
                    crate::racebox_log!(log::Level::Error, "Failed to get peripherals: {e}");
                    on_error(BleError::PeripheralDiscovery(e));
                    continue;
                }
            };

            for p in peripherals {
                if let Ok(Some(props)) = p.properties().await {
                    if let Some(local_name) = props.local_name {
                        crate::racebox_log!(log::Level::Debug, "Peripheral found: {local_name}");
                        if local_name.starts_with("RaceBox Micro") {
                            crate::racebox_log!(log::Level::Info, "RaceBox Micro device found: {local_name}");
                            if let Err(e) = p.connect().await {
                                crate::racebox_log!(log::Level::Error, "Failed to connect to device: {e}");
                                on_error(BleError::Connection(e));
                                continue;
                            }

                            crate::racebox_log!(log::Level::Info, "Connected to device: {local_name}");

                            if let Err(e) = p.discover_services().await {
                                crate::racebox_log!(log::Level::Error, "Failed to discover services: {e}");
                                on_error(BleError::ServiceDiscovery(e));
                                continue;
                            }

                            crate::racebox_log!(log::Level::Debug, "Services discovered");

                            // Log all available services
                            let services = p.services();
                            crate::racebox_log!(log::Level::Debug, "Available services: {:?}", services.iter().map(|s| s.uuid.to_string()).collect::<Vec<_>>());

                            // Find the UART service
                            let uart_service = services.iter().find(|s| s.uuid.to_string().to_uppercase() == UART_SERVICE_UUID.to_uppercase());
                            if let Some(service) = uart_service {
                                crate::racebox_log!(log::Level::Info, "UART service found: {}", service.uuid);
                                // Log all characteristics in the UART service
                                crate::racebox_log!(log::Level::Debug, "UART service characteristics: {:?}", service.characteristics.iter().map(|c| c.uuid.to_string()).collect::<Vec<_>>());
                                // Find the TX characteristic within the UART service
                                let tx = service.characteristics.iter().find(|c| c.uuid.to_string().to_uppercase() == TX_CHAR_UUID.to_uppercase());
                                let tx = match tx {
                                    Some(c) => {
                                        crate::racebox_log!(log::Level::Debug, "TX characteristic found in UART service");
                                        c
                                    },
                                    None => {
                                        crate::racebox_log!(log::Level::Error, "TX characteristic not found in UART service");
                                        on_error(BleError::CharacteristicNotFound);
                                        continue;
                                    }
                                };

                                if let Err(e) = p.subscribe(tx).await {
                                    crate::racebox_log!(log::Level::Error, "Failed to subscribe to notifications: {e}");
                                    on_error(BleError::Subscription(e));
                                    continue;
                                }

                                crate::racebox_log!(log::Level::Info, "Subscribed to notifications");

                                let mut notifications = match p.notifications().await {
                                    Ok(n) => n,
                                    Err(e) => {
                                        crate::racebox_log!(log::Level::Error, "Failed to get notifications: {e}");
                                        on_error(BleError::NotificationSetup(e));
                                        continue;
                                    }
                                };

                                connected = true;
                                crate::racebox_log!(log::Level::Info, "Listening for notifications");
                                while let Some(data) = notifications.next().await {
                                    //crate::racebox_log!(log::Level::Trace, "Notification received: {:x?}", data.value);
                                    if let Some(parsed) = parse_packet(&data.value) {
                                        //crate::racebox_log!(log::Level::Debug, "Parsed RaceBox data: {:?}", parsed);
                                        on_data(parsed);
                                    } else {
                                        crate::racebox_log!(log::Level::Warn, "Failed to parse RaceBox packet");
                                    }
                                }
                            } else {
                                crate::racebox_log!(log::Level::Error, "UART service not found");
                                on_error(BleError::CharacteristicNotFound);
                                continue;
                            }
                        }
                    }
                }
            }
        }
    });
}
