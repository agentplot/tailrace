//! The environment's `nix_cache` table, read at every operation of the
//! target: `attic`, the cache as attic's own command names it, and
//! `closures`, the outputs whose artifacts name closures, and no other
//! key, so no credential sits in the repository's file (45, rule
//! core.21; design D7).

use std::collections::BTreeSet;
use std::fmt;

use serde_json::Value;
use timber_seams::{Answer, Environment, Not, OutputName, Source};

use super::PROFILE;

/// The key naming the cache.
const ATTIC: &str = "attic";

/// The key naming the closure outputs.
const CLOSURES: &str = "closures";

/// The form attic's own command names a cache in.
pub(super) const CACHE_FORM: &str = "<server>:<cache>";

/// What an environment's table names: the cache, `<server>:<cache>`,
/// and one or more closure outputs, each once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Table {
    pub attic: String,
    pub closures: Vec<OutputName>,
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
    /// does not read, an `attic` that is not `<server>:<cache>`, or a
    /// `closures` that names no output, or one twice.
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
                    "it is {}, and the {PROFILE} target reads a table naming {ATTIC} and \
                     {CLOSURES}",
                    environment.settings
                )),
            ));
        };
        if let Some(key) = settings.keys().find(|k| *k != ATTIC && *k != CLOSURES) {
            return Err(wanting(
                key,
                format!(
                    "the {PROFILE} target reads {ATTIC} and {CLOSURES} and no setting {key}; \
                     attic reads the cache's token from its own configuration on the host that \
                     pushes"
                ),
            ));
        }
        let attic = match settings.get(ATTIC) {
            None => {
                return Err(wanting(
                    ATTIC,
                    format!("the table names no {ATTIC}, the cache as {CACHE_FORM}"),
                ))
            }
            Some(Value::String(attic)) => attic,
            Some(other) => {
                return Err(wanting(
                    ATTIC,
                    format!("{ATTIC} is {other}, and names the cache as {CACHE_FORM}"),
                ))
            }
        };
        if let Some(problem) = cache_problem(attic) {
            return Err(wanting(
                ATTIC,
                format!(
                    "{ATTIC} is {attic:?}, and attic's own command names a cache {CACHE_FORM}: \
                     {problem}"
                ),
            ));
        }
        let closures = match settings.get(CLOSURES) {
            None => {
                return Err(wanting(
                    CLOSURES,
                    format!("the table names no {CLOSURES}, the outputs that name closures"),
                ))
            }
            Some(Value::Array(closures)) => closures,
            Some(other) => {
                return Err(wanting(
                    CLOSURES,
                    format!("{CLOSURES} is {other}, and is a list of output names"),
                ))
            }
        };
        if closures.is_empty() {
            return Err(wanting(
                CLOSURES,
                format!("{CLOSURES} names no output, and names one or more"),
            ));
        }
        let mut seen = BTreeSet::new();
        let mut outputs = Vec::new();
        for closure in closures {
            let name = match closure {
                Value::String(name) if !name.is_empty() => name,
                other => {
                    return Err(wanting(
                        CLOSURES,
                        format!("{CLOSURES} holds {other}, which is no output's name"),
                    ))
                }
            };
            if !seen.insert(name) {
                return Err(wanting(CLOSURES, format!("{CLOSURES} names {name} twice")));
            }
            outputs.push(OutputName::new(name.as_str()));
        }
        Ok(Table {
            attic: attic.clone(),
            closures: outputs,
        })
    }
}

/// What keeps `attic` from naming a cache as attic's own command names
/// one: no `:`, nothing before it or after it, a second one, or a
/// server beginning `-`, which attic would read as an option.
fn cache_problem(attic: &str) -> Option<&'static str> {
    let Some((server, cache)) = attic.split_once(':') else {
        return Some("it names no `:`");
    };
    if server.is_empty() {
        Some("it names no server before its `:`")
    } else if cache.is_empty() {
        Some("it names no cache after its `:`")
    } else if cache.contains(':') {
        Some("it names more than one `:`")
    } else if server.starts_with('-') {
        Some("its server begins `-`, which attic reads as an option")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use timber_seams::{ChannelId, ChannelName, ComponentName, EnvironmentName, Seam};

    use super::*;

    fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    fn cache(settings: Value) -> Environment {
        Environment {
            name: EnvironmentName::new("cache"),
            channel: ChannelId::new(
                Some(ComponentName::new("swancloud-srv")),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings,
        }
    }

    /// The key `settings` is unreadable for, from `what`, and its words.
    fn wanting(settings: Value) -> (String, String) {
        match Table::read(&cache(settings.clone()), &source()) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                (u.what, u.said.expect("the target's words"))
            }
            other => panic!("{settings} answered {other:?}"),
        }
    }

    fn setting(key: &str) -> String {
        format!("setting {key} of environment cache of swancloud-srv")
    }

    #[test]
    fn the_cache_and_its_closures_are_read() {
        let table = Table::read(
            &cache(json!({ "attic": "fleet:swancloud", "closures": ["swancloud-srv", "notes"] })),
            &source(),
        )
        .unwrap();
        assert_eq!(table.attic, "fleet:swancloud");
        assert_eq!(
            table.closures,
            [OutputName::new("swancloud-srv"), OutputName::new("notes")]
        );
    }

    #[test]
    fn a_token_in_the_table_is_unreadable_naming_the_environment_and_the_key() {
        let (what, said) = wanting(json!({
            "attic": "fleet:swancloud",
            "closures": ["swancloud-srv"],
            "token": "eyJhbGciOi",
        }));
        assert_eq!(what, setting("token"));
        assert!(said.contains("no setting token"), "{said}");
        assert!(!said.contains("eyJhbGciOi"), "{said}");
    }

    #[test]
    fn a_cache_with_no_server_is_unreadable_saying_it_is_server_and_cache() {
        let (what, said) = wanting(json!({ "attic": "swancloud", "closures": ["swancloud-srv"] }));
        assert_eq!(what, setting("attic"));
        assert!(said.contains("<server>:<cache>"), "{said}");
        assert!(said.ends_with("it names no `:`"), "{said}");
    }

    #[test]
    fn a_cache_attic_cannot_name_is_unreadable_saying_why() {
        for (attic, problem) in [
            (":swancloud", "it names no server before its `:`"),
            ("fleet:", "it names no cache after its `:`"),
            ("fleet:swan:cloud", "it names more than one `:`"),
            (
                "-fleet:swancloud",
                "its server begins `-`, which attic reads as an option",
            ),
        ] {
            let (what, said) = wanting(json!({ "attic": attic, "closures": ["swancloud-srv"] }));
            assert_eq!(what, setting("attic"));
            assert!(said.ends_with(problem), "{attic}: {said}");
        }
        let (what, said) = wanting(json!({ "attic": 3, "closures": ["swancloud-srv"] }));
        assert_eq!(what, setting("attic"));
        assert!(said.starts_with("attic is 3"), "{said}");
    }

    #[test]
    fn an_empty_closures_is_unreadable_naming_it() {
        let (what, said) = wanting(json!({ "attic": "fleet:swancloud", "closures": [] }));
        assert_eq!(what, setting("closures"));
        assert!(said.contains("names no output"), "{said}");
    }

    #[test]
    fn closures_that_are_no_list_of_names_each_once_are_unreadable_naming_it() {
        for (closures, words) in [
            (json!("swancloud-srv"), "closures is \"swancloud-srv\""),
            (json!([""]), "closures holds \"\""),
            (json!([4]), "closures holds 4"),
            (
                json!(["swancloud-srv", "swancloud-srv"]),
                "closures names swancloud-srv twice",
            ),
        ] {
            let (what, said) = wanting(json!({ "attic": "fleet:swancloud", "closures": closures }));
            assert_eq!(what, setting("closures"));
            assert!(said.starts_with(words), "{said}");
        }
    }

    #[test]
    fn a_table_missing_a_key_is_unreadable_naming_it() {
        let (what, _) = wanting(json!({ "closures": ["swancloud-srv"] }));
        assert_eq!(what, setting("attic"));
        let (what, _) = wanting(json!({ "attic": "fleet:swancloud" }));
        assert_eq!(what, setting("closures"));
    }

    #[test]
    fn settings_that_are_no_table_are_unreadable_naming_the_environment() {
        let (what, said) = wanting(json!("fleet:swancloud"));
        assert_eq!(
            what,
            "the nix_cache table of environment cache of swancloud-srv"
        );
        assert!(said.starts_with("it is \"fleet:swancloud\""), "{said}");
    }
}
