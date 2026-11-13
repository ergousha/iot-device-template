//! The product's own logic, kept free of ESP-IDF so it runs and is tested on
//! the development host:
//!
//! ```sh
//! RUSTUP_TOOLCHAIN=stable cargo test -p logic --target host-tuple
//! ```
//!
//! The example: the product's named shadow carries `{"interval_s": N}`, and
//! while the platform's telemetry switch is on the device publishes one
//! small reading every N seconds over Basic Ingest. Replace it with your
//! product's logic; keep the firmware crate (`src/`) a thin shell around it.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Seconds between readings unless the shadow says otherwise.
pub const DEFAULT_INTERVAL_S: u32 = 300;
/// The shortest interval accepted: one message a minute is plenty, and the
/// platform's limiter allows ten.
pub const MIN_INTERVAL_S: u32 = 60;
pub const MAX_INTERVAL_S: u32 = 24 * 3600;

/// What the product's named shadow configures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub interval_s: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_s: DEFAULT_INTERVAL_S,
        }
    }
}

/// The outcome of applying desired state.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    pub settings: Settings,
    /// The `reported` object to publish: what the device now runs, and
    /// `error` for anything it refused (null clears an earlier one).
    pub reported: Value,
}

/// Applies a desired-state object (the `state` of an `update/delta`, or
/// `state.desired` of a `get/accepted`). Unknown keys are ignored; a bad
/// value keeps the current setting and is reported.
pub fn apply_desired(current: Settings, desired: &Value) -> Applied {
    let mut settings = current;
    let mut error = Value::Null;
    if let Some(v) = desired.get("interval_s") {
        match v.as_u64().and_then(|n| u32::try_from(n).ok()) {
            Some(n) if (MIN_INTERVAL_S..=MAX_INTERVAL_S).contains(&n) => settings.interval_s = n,
            _ => {
                error = json!(format!(
                    "interval_s must be a whole number of seconds from {MIN_INTERVAL_S} to {MAX_INTERVAL_S}"
                ))
            }
        }
    }
    Applied {
        settings,
        reported: json!({ "interval_s": settings.interval_s, "error": error }),
    }
}

/// The desired-state object in a shadow message: `state` of a delta,
/// `state.desired` of a full document.
pub fn desired_of(message: &Value) -> Option<&Value> {
    let state = message.get("state")?;
    Some(state.get("desired").unwrap_or(state))
}

/// One reading. Small on purpose: AWS bills per 5 KB and per rule action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reading<'a> {
    pub thing: &'a str,
    pub fw: &'a str,
    pub up_s: u64,
    pub heap: u32,
}

/// The reading as JSON, or `None` if it would exceed `max_bytes`.
pub fn encode(reading: &Reading<'_>, max_bytes: usize) -> Option<Vec<u8>> {
    let bytes = serde_json::to_vec(reading).ok()?;
    (bytes.len() <= max_bytes).then_some(bytes)
}

/// The Basic Ingest topic of the product's `<product>_readings` rule
/// (cloud/): no messaging fee, straight to the rule.
pub fn readings_topic(product: &str, thing: &str) -> String {
    format!("$aws/rules/{product}_readings/{thing}")
}

/// Whether a reading is due at `now_s`, given the last one.
pub fn due(last_s: Option<u64>, now_s: u64, settings: Settings) -> bool {
    last_s.is_none_or(|last| now_s.saturating_sub(last) >= u64::from(settings.interval_s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_interval_is_applied_and_reported() {
        let a = apply_desired(Settings::default(), &json!({"interval_s": 120, "other": 1}));
        assert_eq!(a.settings.interval_s, 120);
        assert_eq!(a.reported, json!({"interval_s": 120, "error": null}));
    }

    #[test]
    fn a_bad_interval_keeps_the_current_one_and_says_why() {
        for bad in [json!(10), json!(-5), json!("60"), json!(1e12), json!(90000)] {
            let a = apply_desired(Settings { interval_s: 600 }, &json!({ "interval_s": bad }));
            assert_eq!(a.settings.interval_s, 600, "{bad}");
            assert!(a.reported["error"]
                .as_str()
                .unwrap()
                .contains("60 to 86400"));
        }
    }

    #[test]
    fn nothing_desired_reports_the_current_state() {
        let a = apply_desired(Settings::default(), &json!({}));
        assert_eq!(
            a.reported,
            json!({"interval_s": DEFAULT_INTERVAL_S, "error": null})
        );
    }

    #[test]
    fn desired_comes_from_deltas_and_full_documents() {
        let delta = json!({"version": 3, "state": {"interval_s": 90}});
        let doc = json!({"state": {"desired": {"interval_s": 90}, "reported": {"interval_s": 60}}});
        assert_eq!(desired_of(&delta), Some(&json!({"interval_s": 90})));
        assert_eq!(desired_of(&doc), Some(&json!({"interval_s": 90})));
        assert_eq!(desired_of(&json!({"code": 404})), None);
    }

    #[test]
    fn readings_are_small_and_bounded() {
        let r = Reading {
            thing: "AABBCCDDEEFF",
            fw: "0.1.0",
            up_s: 42,
            heap: 123_456,
        };
        let bytes = encode(&r, 5120).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&bytes).unwrap(),
            json!({"thing": "AABBCCDDEEFF", "fw": "0.1.0", "up_s": 42, "heap": 123456})
        );
        assert!(encode(&r, 10).is_none());
        assert_eq!(
            readings_topic("demo", "AABBCCDDEEFF"),
            "$aws/rules/demo_readings/AABBCCDDEEFF"
        );
    }

    #[test]
    fn readings_fall_due_by_the_interval() {
        let s = Settings { interval_s: 60 };
        assert!(due(None, 5, s));
        assert!(!due(Some(100), 159, s));
        assert!(due(Some(100), 160, s));
    }
}
