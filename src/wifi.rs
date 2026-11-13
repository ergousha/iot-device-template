//! Wi-Fi station. The credentials come from the bench (`ergo commission
//! --wifi-ssid`), stored by the SDK in NVS `platform`; images carry none.
//! Without them the device waits for the bench console rather than
//! restarting (a restart loop would cut `ergo commission` off).

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

/// Association attempts before the caller restarts.
const CONNECT_ATTEMPTS: u32 = 3;

pub fn start<'d>(
    modem: Modem<'d>,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    platform: &device_sdk::Platform,
) -> Result<BlockingWifi<EspWifi<'d>>> {
    let (ssid, psk) = loop {
        if let Some(credentials) = platform.wifi() {
            break credentials;
        }
        log::warn!("no Wi-Fi credentials; waiting for `ergo commission` on the serial console");
        std::thread::sleep(Duration::from_secs(5));
    };

    let esp_wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs)).context("EspWifi::new")?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop).context("BlockingWifi::wrap")?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .as_str()
            .try_into()
            .map_err(|_| anyhow!("SSID over 32 bytes"))?,
        password: psk
            .as_str()
            .try_into()
            .map_err(|_| anyhow!("PSK over 64 bytes"))?,
        auth_method: if psk.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        },
        ..Default::default()
    }))?;
    wifi.start().context("Wi-Fi start")?;
    log::info!("connecting to Wi-Fi {ssid}");

    // The first association after a software reset times out now and then;
    // a restart of an image not yet marked valid would roll it back.
    let mut attempt = 1;
    loop {
        match wifi.connect() {
            Ok(()) => break,
            Err(e) if attempt < CONNECT_ATTEMPTS => {
                log::warn!("Wi-Fi attempt {attempt} of {CONNECT_ATTEMPTS} failed: {e}");
                let _ = wifi.disconnect();
                std::thread::sleep(Duration::from_secs(3));
                attempt += 1;
            }
            Err(e) => return Err(e).context("Wi-Fi connect"),
        }
    }
    wifi.wait_netif_up().context("Wi-Fi netif")?;
    log::info!("Wi-Fi up: {:?}", wifi.wifi().sta_netif().get_ip_info()?);
    Ok(wifi)
}
