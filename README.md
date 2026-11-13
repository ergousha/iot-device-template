# iot-device-template

The starting point for a new product on the Ergousha IoT platform
([iot-platform](https://github.com/ergousha/iot-platform),
[iot-device-sdk](https://github.com/ergousha/iot-device-sdk)): ESP32 or
ESP32-S3 firmware in Rust (ESP-IDF), on Wi-Fi, with its device key generated
on the device. Out of the box it enrols, connects, updates itself through
signed OTA releases, reports the `platform` shadow, applies its own named
shadow (`{"interval_s": N}`), and publishes a small reading every N seconds
while the platform's telemetry switch is on.

## A new product in under an hour

1. **Use this template** (public, under the owner's account), and clone it
   next to `iot-platform` and `iot-device-sdk`.
2. Put it on the platform (the owner's AWS session; `gh` logged in):

   ```sh
   aws-vault exec <owner> -- cargo run --manifest-path ../iot-device-sdk/Cargo.toml -p ergo -- \
       product init <id> --description "<one line>" --infra ../iot-platform [--chip esp32]
   ```

   It sets the product id and chip, adds the product to the platform (a pull
   request on iot-platform and its `terraform apply`), writes the platform's
   endpoint, template and `cloud/` backend here, sets the repository
   variables, settings and ruleset, and last creates the signing keys and
   Environment `release`. It ends with a `chore:` pull request: merge it.
3. Make the product yours, then merge a `feat:` commit: that is release
   `0.1.0`, built, signed and uploaded by the release workflow.
4. Flash a board with the release's `factory.bin`
   (`espflash write-bin 0x0 factory.bin`) and commission it as the canary:

   ```sh
   WIFI_PSK=... aws-vault exec <owner> -- cargo run --manifest-path ../iot-device-sdk/Cargo.toml -p ergo -- \
       commission --product <id> --port /dev/cu.usbmodem1101 --canary --wifi-ssid "<ssid>" --wifi-psk-env WIFI_PSK
   ```

   Every later `feat:`/`fix:` merge reaches it, then the fleet, with no
   clicks (PLATFORM.md §9).

## Layout

| Path | What |
|---|---|
| `platform.toml` | the product on the platform: id, chip, MAC type, key storage, endpoint, template, limits, signing keys, frozen partitions |
| `src/` | the firmware: Wi-Fi, clock, the SDK session (`main.rs`), the product loop (`app.rs`) |
| `logic/` | the product's logic as pure functions, tested on the host |
| `keys/`, `certs/` | signing public keys; the Amazon root CA |
| `cloud/` | optional AWS resources of the product (Terraform, applied by the owner) |
| `web/` | optional web app |
| `.github/workflows/` | `ci.yml` (firmware, host tests, `cloud/`), `release.yml` (the platform pipeline), `zizmor.yml`, `audit.yml` |

## Develop

```sh
. ~/export-esp.sh                     # espup's Xtensa toolchain
cargo build --release
cargo run --release                   # flash and monitor
RUSTUP_TOOLCHAIN=stable cargo test -p logic --target host-tuple
```

The contract, the cost rules and the security baseline are in
[PLATFORM.md](PLATFORM.md). Commit messages are conventional: `feat:`
releases a minor version, `fix:` a patch, anything else nothing.

## License

MIT, see [LICENSE](LICENSE).
