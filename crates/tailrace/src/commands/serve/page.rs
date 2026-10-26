//! The serving process's pages (57, 59, 132; design D8): each one HTML
//! document, its styles inline and no script, rendered from one answer
//! of the core and nothing else (rule core.31). Every string is escaped,
//! and every refusal and unreadable answer is shown with the source's
//! own words in a `<pre>` of their own.

use std::fmt::Write as _;

use tailrace_core::{
    Channel, ChannelStatus, Channels, EnvironmentCounts, HeadAt, Overview, Repositories,
    RepositoryName, Row, Status,
};
use tailrace_seams::{Answer, ComponentName, Digest, Moves, Not, Promotion, Read};

use crate::commands::status::satisfied;
use crate::render::utc;

/// `text` with every character HTML gives a meaning escaped.
pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            c => escaped.push(c),
        }
    }
    escaped
}

const STYLE: &str = "body{font-family:system-ui,sans-serif;margin:2rem;max-width:72rem;color:#1b1b1b;background:#fff}\
table{border-collapse:collapse;margin:0.5rem 0 1.5rem}\
th,td{text-align:left;padding:0.25rem 0.75rem;border-bottom:1px solid #ddd;vertical-align:top}\
pre{white-space:pre-wrap;background:#f4f4f4;padding:0.5rem;margin:0.25rem 0}\
.not{color:#8a1c1c}.chain{margin:0;padding-left:1.25rem;color:#444}\
@media (prefers-color-scheme:dark){body{color:#e8e8e8;background:#161616}th,td{border-color:#333}pre{background:#222}.not{color:#f09b9b}.chain{color:#bbb}}";

/// One HTML document titled `title` holding `body`.
fn document(title: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n{body}</body>\n</html>\n",
        escape(title)
    )
}

/// `not` as a person reads it: tailrace's reason, then the source's own
/// words, unchanged, in a `<pre>` of their own (59).
pub fn not(not: &Not) -> String {
    let mut reason = not.clone();
    let said = match &mut reason {
        Not::Absent(_) => None,
        Not::Refused(r) => r.said.take(),
        Not::Unreadable(u) => u.said.take(),
    };
    let mut html = format!("<span class=\"not\">{}</span>", escape(&reason.to_string()));
    if let Some(said) = said {
        let _ = write!(html, "<pre>{}</pre>", escape(&said));
    }
    html
}

/// A link to the page of `repository`.
fn link(repository: &RepositoryName) -> String {
    format!(
        "<a href=\"{}/status\">{}</a>",
        escape(&repository.path()),
        escape(&repository.to_string())
    )
}

/// 132: the root's page, from every repository the host knows: one row
/// each, the repository linked to its own page, every channel's head,
/// and the environments' four counts or the answer given in their place;
/// a row's own answer spanning the row.
pub fn root(answer: &Answer<Repositories>) -> String {
    let mut body = String::from("<h1>Every repository on this host</h1>\n");
    match answer {
        Err(not) => {
            let _ = writeln!(body, "<p>{}</p>", self::not(not));
        }
        Ok(repositories) => {
            let _ = writeln!(body, "<p>As of {}</p>", utc(repositories.asked_at));
            if repositories.repositories.is_empty() {
                body.push_str(
                    "<p>No checkout on this host has named a repository yet. \
                     <code>tailrace status --page</code>, run in a checkout, names its \
                     repository here.</p>\n",
                );
            } else {
                body.push_str(
                    "<table>\n<tr><th>repository</th><th>channels</th><th>environments</th></tr>\n",
                );
                for row in &repositories.repositories {
                    match row {
                        Ok(row) => row_html(&mut body, row),
                        Err(not) => {
                            let _ = writeln!(
                                body,
                                "<tr><td colspan=\"3\">{}</td></tr>",
                                self::not(not)
                            );
                        }
                    }
                }
                body.push_str("</table>\n");
            }
        }
    }
    document("tailrace on this host", &body)
}

fn row_html(body: &mut String, row: &Row) {
    let heads = match &row.heads {
        Err(not) => self::not(not),
        Ok(heads) if heads.is_empty() => "no channel declared".to_string(),
        Ok(heads) => {
            let mut list = String::from("<ul class=\"chain\">");
            for head in heads {
                let at = match &head.at {
                    Ok(HeadAt::Empty) => "empty".to_string(),
                    Ok(HeadAt::At { version, index }) => format!(
                        "{} <code>{}</code>",
                        escape(version.as_str()),
                        escape(index.as_str())
                    ),
                    Err(not) => self::not(not),
                };
                let _ = write!(
                    list,
                    "<li>{}/{} {at}</li>",
                    escape(head.component.as_str()),
                    escape(head.channel.as_str())
                );
            }
            list.push_str("</ul>");
            list
        }
    };
    let _ = writeln!(
        body,
        "<tr><td>{}</td><td>{heads}</td><td>{}</td></tr>",
        link(&row.repository),
        environments(&row.environments)
    );
}

/// 132: `repository`'s page, from its status answer: its channels by
/// component, each with its version or empty or its own answer and its
/// chain newest first, then the environments and what this host
/// published.
pub fn repository(repository: &RepositoryName, answer: &Answer<Overview>) -> String {
    let mut body = String::from("<p><a href=\"/\">Every repository on this host</a></p>\n");
    let _ = writeln!(body, "<h1>{}</h1>", escape(&repository.to_string()));
    match answer {
        Err(not) => {
            let _ = writeln!(body, "<p>{}</p>", self::not(not));
        }
        Ok(overview) => overview_html(&mut body, overview),
    }
    document(&format!("{repository} status"), &body)
}

fn overview_html(body: &mut String, overview: &Overview) {
    let asked = utc(overview.asked_at);
    match &overview.channels {
        Ok(channels) => {
            let _ = writeln!(
                body,
                "<p>{} at <code>{}</code>, as of {asked}</p>",
                escape(overview.branch.as_str()),
                escape(channels.revision.as_str())
            );
            landing_html(body, &channels.landing);
            channels_html(body, channels);
        }
        Err(not) => {
            let _ = writeln!(
                body,
                "<p>{}, as of {asked}</p>\n<h2>Channels</h2>\n<p>{}</p>",
                escape(overview.branch.as_str()),
                self::not(not)
            );
        }
    }
    let _ = writeln!(
        body,
        "<h2>Environments</h2>\n<p>{}</p>",
        environments(&overview.environments)
    );
    body.push_str("<h2>Published from this host</h2>\n");
    published(body, &overview.published);
}

/// 77: how the landing read the branch's head, as the command line
/// says it: nothing where it read itself, and where it read the host's
/// copy, when that was fetched and the remote's words, unchanged, in a
/// `<pre>` of their own (59; design D6).
fn landing_html(body: &mut String, read: &Read) {
    if let Read::Kept { at, said } = read {
        let _ = writeln!(
            body,
            "<p>landing: as last fetched {}</p>\n<pre>{}</pre>",
            utc(*at),
            escape(said)
        );
    }
}

/// Each component's channels in a table of its own, in the order the
/// policy declares them.
fn channels_html(body: &mut String, channels: &Channels) {
    let mut components: Vec<(ComponentName, Vec<&ChannelStatus>)> = Vec::new();
    for channel in &channels.channels {
        let component = channel.declared.channel.component_name();
        match components.iter_mut().find(|(c, _)| *c == component) {
            Some((_, of)) => of.push(channel),
            None => components.push((component, vec![channel])),
        }
    }
    if components.is_empty() {
        body.push_str("<h2>Channels</h2>\n<p>The policy declares no channel.</p>\n");
    }
    for (component, of) in components {
        let _ = writeln!(
            body,
            "<h2>{}</h2>\n<table>\n<tr><th>channel</th><th>bound to</th><th>at</th></tr>",
            escape(component.as_str())
        );
        for channel in of {
            channel_html(body, channel);
        }
        body.push_str("</table>\n");
    }
}

fn channel_html(body: &mut String, channel: &ChannelStatus) {
    let bound = match &channel.declared.moves {
        Moves::Bound { to } => escape(to.as_str()),
        Moves::EachPublish | Moves::Promotion => String::new(),
    };
    let at = match &channel.at {
        Ok(Channel::Empty) => "empty".to_string(),
        Ok(Channel::At { head, chain, read }) => {
            let mut at = format!(
                "{} <code>{}</code>",
                escape(head.to.version.as_str()),
                escape(head.to.index.as_str())
            );
            if let Read::Kept { at: when, said } = read {
                let _ = write!(
                    at,
                    ", as last read {}<pre>{}</pre>",
                    utc(*when),
                    escape(said)
                );
            }
            at.push_str("<ol class=\"chain\" reversed>");
            for (claim, promotion) in chain.iter().rev() {
                let _ = write!(at, "<li>{}</li>", promoted(claim, promotion));
            }
            at.push_str("</ol>");
            at
        }
        Err(not) => self::not(not),
    };
    let _ = writeln!(
        body,
        "<tr><td>{}</td><td>{bound}</td><td>{at}</td></tr>",
        escape(channel.name().as_str())
    );
}

/// 33, 105: one promotion of a chain, where it moved the channel from,
/// who asked, the policy revision it was judged under with the claims
/// that satisfied it, when, and its claim and its predecessor's.
fn promoted(claim: &Digest, promotion: &Promotion) -> String {
    let from = promotion.from.as_ref().map_or_else(
        || "the first".to_string(),
        |f| {
            format!(
                "from {} <code>{}</code>",
                escape(f.version.as_str()),
                escape(f.index.as_str())
            )
        },
    );
    let after = promotion
        .predecessor
        .as_ref()
        .map_or_else(String::new, |p| {
            format!(" after <code>{}</code>", escape(p.as_str()))
        });
    let satisfied = satisfied(
        promotion
            .policy
            .claims
            .iter()
            .map(|c| format!("<code>{}</code>", escape(c.as_str()))),
    );
    format!(
        "{} <code>{}</code>, {from}, asked by {}, under the policy at <code>{}</code> satisfied by {satisfied}, {}; claim <code>{}</code>{after}",
        escape(promotion.to.version.as_str()),
        escape(promotion.to.index.as_str()),
        escape(promotion.asked_by.as_str()),
        escape(promotion.policy.revision.as_str()),
        utc(promotion.at),
        escape(claim.as_str())
    )
}

/// 98: the environments' four counts, or the part's own answer.
pub fn environments(answer: &Answer<EnvironmentCounts>) -> String {
    match answer {
        Ok(counts) => format!(
            "{} running what they should, {} behind, {} waiting on a person, {} unreadable",
            counts.converged, counts.behind, counts.waiting, counts.unreadable
        ),
        Err(not) => self::not(not),
    }
}

fn published(body: &mut String, answer: &Answer<Status>) {
    let status = match answer {
        Ok(status) => status,
        Err(not) => {
            let _ = writeln!(body, "<p>{}</p>", self::not(not));
            return;
        }
    };
    body.push_str(
        "<table>\n<tr><th>component</th><th>version</th><th>tag</th><th>revision</th>\
         <th>branch</th><th>index</th><th>store</th></tr>\n",
    );
    for component in &status.components {
        for entry in &component.versions {
            let _ = writeln!(
                body,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td>\
                 <td><code>{}</code></td><td>{}</td></tr>",
                escape(component.component.as_str()),
                escape(entry.version.as_str()),
                escape(&entry.tag),
                escape(entry.revision.as_str()),
                escape(entry.branch.as_str()),
                escape(entry.index.as_str()),
                escape(&entry.store.to_string())
            );
        }
    }
    body.push_str("</table>\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_character_html_gives_a_meaning_is_escaped() {
        assert_eq!(
            escape("<script>alert(\"a\" & 'b')</script>"),
            "&lt;script&gt;alert(&quot;a&quot; &amp; &#39;b&#39;)&lt;/script&gt;"
        );
    }
}
