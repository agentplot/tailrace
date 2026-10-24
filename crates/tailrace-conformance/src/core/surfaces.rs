//! The surfaces' group: a surface answers what the one core answers, value
//! for value, as a program reads it (56, 57; design D6).

use serde_json::Value;
use tailrace_seams::{Revision, Version};

use crate::world::FakeWorld;
use crate::{Rig, Scenario};

pub const GROUP: &str = "surfaces";

/// A question every surface puts to the core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Question {
    /// Publish `revision`, every component the landing touched.
    Publish { revision: Revision },
    /// Verify `version` of the one component, under the policy for
    /// acceptance.
    Verify { version: Version },
    /// Status, of every version the host published.
    Status,
}

/// A surface over a world of fakes, made new for each scenario.
pub trait Surface {
    /// The surface, by name.
    fn name(&self) -> String;

    /// The world the surface asks over.
    fn world(&self) -> &FakeWorld;

    /// `question` asked through the surface, its answer as a program
    /// reads it.
    fn ask(&self, question: &Question) -> Value;
}

impl Rig for dyn Surface {
    fn profile(&self) -> String {
        self.name()
    }
}

pub fn scenarios() -> Vec<Scenario<dyn Surface>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
