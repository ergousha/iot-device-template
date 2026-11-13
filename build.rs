//! Build-time configuration from `platform.toml` (iot-device-sdk's
//! platform-build): the `PLATFORM` constant, FW_VERSION, the signing keyring,
//! the partition check and the bootloader for espflash. No AWS access and no
//! secrets: the claim and Wi-Fi reach a unit at the bench (`ergo commission`).

fn main() {
    platform_build::configure();
}
