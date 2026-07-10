// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A non-conforming advisory override (missing the required markers) is rejected
//! by the kernel and returns `Err(ConsumerError::Kernel(..))` with
//! `code == "render"`, `field == "guard"`. Standalone —
//! `cargo run --example guides_guard_override_rejected`.

use garde::Validate;
use prompting_press::{ConsumerError, GuardConfig, Prompt};
use serde::Serialize;
use std::fs;

#[derive(Serialize, Validate)]
struct Ask {
    #[garde(length(min = 1))]
    topic: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    let ask = Prompt::from_yaml(&fs::read_to_string(format!("{dir}/ask.yaml"))?)?;
    let vars = Ask {
        topic: "rivers".into(),
    };

    let bad = GuardConfig {
        enabled: true,
        advisory: Some("Missing the required markers.".into()),
    };
    match ask.render(&vars, None, &bad, false) {
        Err(ConsumerError::Kernel(rows)) => {
            assert_eq!(rows[0].code, "render");
            assert_eq!(rows[0].field, "guard");
        }
        _ => unreachable!("a non-conforming advisory is rejected"),
    }
    Ok(())
}
