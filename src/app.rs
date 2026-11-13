//! The product's loop: its named shadow and its telemetry. The decisions are
//! in the `logic` crate (host-tested); this only moves messages.

use std::time::Duration;

use anyhow::Result;
use device_sdk::core::product::Product;
use device_sdk::core::shadow::Topics;
use device_sdk::{Event, Session};
use logic::{Reading, Settings};

pub fn run(mut session: Session, product: &Product) -> Result<()> {
    let thing = session.thing_name().to_string();
    // The product's named shadow: its id, unless products/<id>.json in
    // iot-platform sets shadow_name.
    let shadow = Topics::new(&thing, product.id);
    let readings = logic::readings_topic(product.id, &thing);
    let mut settings = Settings::default();
    let mut last_reading: Option<u64> = None;

    loop {
        match session.poll(Duration::from_millis(500)) {
            Some(Event::Connected) => {
                // Subscriptions do not survive a lost session: subscribe to
                // the answers this loop acts on (never shadow/#), then ask
                // for the document.
                for topic in shadow.subscriptions() {
                    session.client().subscribe(topic)?;
                }
                session.publish(&shadow.get, b"{}")?;
            }
            Some(Event::Message { topic, data }) if shadow.owns(&topic) => {
                if topic == shadow.get_rejected {
                    // No document yet (404): report what runs.
                    report(
                        &session,
                        &shadow,
                        logic::apply_desired(settings, &serde_json::json!({})),
                    );
                } else if let Some(desired) = serde_json::from_slice(&data)
                    .ok()
                    .as_ref()
                    .and_then(logic::desired_of)
                {
                    let applied = logic::apply_desired(settings, desired);
                    settings = applied.settings;
                    report(&session, &shadow, applied);
                }
            }
            Some(Event::Restart) => device_sdk::restart_into_update(),
            _ => {}
        }

        // Telemetry only while the platform's switch is on (`platform`
        // shadow, desired.telemetry.enabled): it is switched on only when a
        // rule in cloud/ consumes it.
        let now = device_sdk::uptime_s();
        if session.is_connected()
            && session.telemetry_enabled()
            && logic::due(last_reading, now, settings)
        {
            last_reading = Some(now);
            let reading = Reading {
                thing: &thing,
                fw: product.fw_version,
                up_s: now,
                // SAFETY: a plain getter.
                heap: unsafe { esp_idf_svc::sys::esp_get_free_heap_size() },
            };
            match logic::encode(&reading, product.limits.max_message_bytes) {
                Some(payload) => {
                    if let Err(e) = session.publish(&readings, &payload) {
                        log::warn!("reading not sent: {e:#}");
                    }
                }
                None => log::warn!("reading over the message limit; not sent"),
            }
        }
    }
}

fn report(session: &Session, shadow: &Topics, applied: logic::Applied) {
    let doc = serde_json::json!({ "state": { "reported": applied.reported } });
    if let Err(e) = session.publish(&shadow.update, doc.to_string().as_bytes()) {
        log::warn!("shadow report not sent: {e:#}");
    }
}
