//! The environment's `clan` table, read at every operation of the
//! target: `machine`, the machine's name in the fleet's inventory, and
//! `closure`, the component's output whose file names the machine's
//! closure, both required strings, and no other key. The machine is
//! reached as the fleet reaches it, so no host, user, key or credential
//! is read from the table (45, C.10; design D2).

use std::fmt;

use serde_json::Value;
use timber_seams::{Answer, Environment, Not, OutputName, Source};

use super::PROFILE;

/// The key naming the machine.
pub(super) const MACHINE: &str = "machine";

/// The key naming the closure's output.
pub(super) const CLOSURE: &str = "closure";

/// What an environment's table names: the machine and the closure's
/// output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Table {
    pub machine: String,
    pub closure: OutputName,
}

/// An environment as an answer names it: by its name and its component.
pub(super) struct Named<'a>(pub &'a Environment);

impl fmt::Display for Named<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let environment = self.0;
        write!(
            f,
            "environment {} of {}",
            environment.name,
            environment.channel.component_name()
        )
    }
}

impl Table {
    /// `environment`'s settings read as the table, or unreadable naming
    /// the environment and the first key found wanting: one the target
    /// does not read, with the two it reads, or a required one missing,
    /// no string, or naming nothing. A machine's name is letters, digits,
    /// `-` and `_`, beginning with a letter or a digit, as it is a part
    /// of a Nix attribute path and an argument of clan's.
    pub fn read(environment: &Environment, source: &Source) -> Answer<Table> {
        let named = Named(environment);
        let wanting = |key: &str, said: String| {
            Not::unreadable(source, format!("setting {key} of {named}"), Some(said))
        };
        let Some(settings) = environment.settings.as_object() else {
            return Err(Not::unreadable(
                source,
                format!("the {PROFILE} table of {named}"),
                Some(format!(
                    "it is {}, and the {PROFILE} target reads a table naming {MACHINE} and \
                     {CLOSURE}",
                    environment.settings
                )),
            ));
        };
        if let Some(key) = settings.keys().find(|k| *k != MACHINE && *k != CLOSURE) {
            return Err(wanting(
                key,
                format!(
                    "the {PROFILE} target reads {MACHINE} and {CLOSURE} and no setting {key}; \
                     clan reaches the machine as the fleet's own inventory names it, with this \
                     host's own ssh"
                ),
            ));
        }
        let text = |key: &str, what: &str| match settings.get(key) {
            None => Err(wanting(
                key,
                format!("the table names no {key}, which is required: {what}"),
            )),
            Some(Value::String(text)) if !text.is_empty() => Ok(text.clone()),
            Some(other) => Err(wanting(key, format!("{key} is {other}, and names {what}"))),
        };
        let machine = text(MACHINE, "the machine's name in the fleet")?;
        let closure = text(CLOSURE, "the output whose file names the machine's closure")?;
        if let Some(problem) = machine_problem(&machine) {
            return Err(wanting(
                MACHINE,
                format!(
                    "{MACHINE} is {machine:?}, and a machine is named by letters, digits, `-` \
                     and `_`: {problem}"
                ),
            ));
        }
        Ok(Table {
            machine,
            closure: OutputName::new(closure),
        })
    }
}

/// What keeps `machine` from naming a machine: a first character that is
/// no letter or digit, or another that is no letter, digit, `-` or `_`.
fn machine_problem(machine: &str) -> Option<String> {
    if !machine.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return Some("it begins with no letter or digit".into());
    }
    machine
        .chars()
        .find(|&c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .map(|c| format!("it holds `{c}`"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use timber_seams::{ChannelId, ChannelName, ComponentName, EnvironmentName, Seam};

    use super::*;

    fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    fn srv(settings: Value) -> Environment {
        Environment {
            name: EnvironmentName::new("srv"),
            channel: ChannelId::new(
                Some(ComponentName::new("swancloud-srv")),
                ChannelName::new("stable"),
            ),
            waits: true,
            settings,
        }
    }

    /// The key `settings` is unreadable for, from `what`, and its words.
    fn wanting(settings: Value) -> (String, String) {
        match Table::read(&srv(settings.clone()), &source()) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                (u.what, u.said.expect("the target's words"))
            }
            other => panic!("{settings} answered {other:?}"),
        }
    }

    fn setting(key: &str) -> String {
        format!("setting {key} of environment srv of swancloud-srv")
    }

    // Requirement: The target's table names the machine and its closure,
    // and nothing else

    #[test]
    fn both_keys_are_read() {
        let table = Table::read(
            &srv(json!({ "machine": "swancloud-srv", "closure": "swancloud-srv" })),
            &source(),
        )
        .unwrap();
        assert_eq!(
            table,
            Table {
                machine: "swancloud-srv".into(),
                closure: OutputName::new("swancloud-srv"),
            }
        );
    }

    #[test]
    fn a_host_address_in_the_table_is_unreadable_naming_it_and_the_two_the_table_reads() {
        let (what, said) = wanting(json!({
            "machine": "swancloud-srv",
            "closure": "swancloud-srv",
            "host": "root@192.168.1.172",
        }));
        assert_eq!(what, setting("host"));
        assert!(
            said.starts_with("the clan target reads machine and closure and no setting host"),
            "{said}"
        );
        assert!(!said.contains("192.168.1.172"), "{said}");
    }

    #[test]
    fn no_closure_named_is_unreadable_naming_closure_as_required() {
        let (what, said) = wanting(json!({ "machine": "swancloud-srv" }));
        assert_eq!(what, setting("closure"));
        assert!(
            said.starts_with("the table names no closure, which is required"),
            "{said}"
        );
        let (what, _) = wanting(json!({ "closure": "swancloud-srv" }));
        assert_eq!(what, setting("machine"));
    }

    #[test]
    fn a_key_that_is_no_name_is_unreadable_naming_it() {
        for (settings, key, words) in [
            (
                json!({ "machine": 3, "closure": "swancloud-srv" }),
                "machine",
                "machine is 3",
            ),
            (
                json!({ "machine": "swancloud-srv", "closure": "" }),
                "closure",
                "closure is \"\"",
            ),
            (
                json!({ "machine": "-swancloud", "closure": "swancloud-srv" }),
                "machine",
                "machine is \"-swancloud\"",
            ),
            (
                json!({ "machine": "srv.local", "closure": "swancloud-srv" }),
                "machine",
                "machine is \"srv.local\"",
            ),
        ] {
            let (what, said) = wanting(settings);
            assert_eq!(what, setting(key));
            assert!(said.starts_with(words), "{said}");
        }
    }

    #[test]
    fn settings_that_are_no_table_are_unreadable_naming_the_environment() {
        let (what, said) = wanting(json!("swancloud-srv"));
        assert_eq!(what, "the clan table of environment srv of swancloud-srv");
        assert!(said.starts_with("it is \"swancloud-srv\""), "{said}");
    }
}
