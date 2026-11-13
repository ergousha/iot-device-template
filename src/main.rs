//! A product on the Ergousha IoT platform (iot-device-sdk; the contract is
//! the SDK's PLATFORM.md).
//!
//! 1. The SDK counts the boot, makes sure the device has its P-256 key and
//!    starts the bench console (`ergo commission`).
//! 2. Wi-Fi (credentials from the bench), then the clock (TLS checks dates).
//! 3. The identity: stored, or enrolment with the claim written at the bench.
//! 4. The session: AWS IoT with OTA, the `platform` shadow and the limiter
//!    handled by the SDK; the product's own loop in `app`.

mod app;
mod wifi;

use std::time::{Duration, Instant};

use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::EspSntp;

// `PLATFORM`: this product as platform.toml describes it (platform-build).
include!(concat!(env!("OUT_DIR"), "/platform.rs"));

/// How long a valid image waits after a failure before it restarts: long
/// enough that a lasting failure (AWS unreachable) is no tight boot loop.
const RESTART_AFTER_FAILURE: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // A new image that never reaches AWS IoT is restarted after
    // device_sdk::VERIFY_DEADLINE, and the bootloader rolls it back.
    device_sdk::start_rollback_guard();

    let result = run();
    if let Err(e) = &result {
        // Returning would park the device with no MQTT, so no Jobs and no
        // way to deliver a fix. An unverified image restarts at once (the
        // bootloader restores the previous one); a valid one after a pause.
        if device_sdk::running_unverified() {
            log::error!("{e:#}; restarting into the previous image");
            std::thread::sleep(Duration::from_secs(2));
        } else {
            log::error!("{e:#}; restarting in {} s", RESTART_AFTER_FAILURE.as_secs());
            std::thread::sleep(RESTART_AFTER_FAILURE);
        }
        esp_idf_svc::hal::reset::restart();
    }
    result
}

fn run() -> Result<()> {
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Before the network: a soft key generated now uses the RNG's
    // radio-off entropy source.
    let platform = device_sdk::Platform::init(&PLATFORM, nvs.clone())?;

    let _wifi = wifi::start(peripherals.modem, sysloop, nvs, &platform)?;

    let _sntp = EspSntp::new_default()?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while !device_sdk::clock_is_set() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(250));
    }
    if !device_sdk::clock_is_set() {
        anyhow::bail!("no time from SNTP in 30 s");
    }

    let identity = platform.identity()?;
    let session = platform.connect(&identity, &device_sdk::ConnectOptions::default())?;
    app::run(session, &PLATFORM)
}
