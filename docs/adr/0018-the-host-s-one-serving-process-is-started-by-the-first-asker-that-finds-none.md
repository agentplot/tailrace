---
number: 18
title: The host's one serving process is started by the first asker that finds none, and answers the repositories a checkout named
status: accepted
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# The host's one serving process is started by the first asker that finds none, and answers the repositories a checkout named

Status: accepted by the user on 2026-10-27, on their card.

Deciders: proposed by the status-on-any-machine unit of phase 1's bolt 4, which shows a repository's channels and why each is where it is on any machine, one that never published included, on the command line and as a page (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), with the address, the path and how the process learns a repository ruled by madswan-design at `agentplot/blueprints` dbe7ca0, how the process starts and ends ruled on the user's word at 85bbf54, and the root page ruled on the user's word at 157c828, for the user to accept with the change.
Sources: statement clauses 49, 56 to 63, 98, 114, 115, 118, 132, as amended at `agentplot/blueprints` 85bbf54 and 157c828; sluice clause 129; ADRs 0006, 0013; rules core.1, core.18, core.19, core.20, core.31, core.32, core.36; Rust 1.98's `std::net` and `std::fs::File::try_lock`; the change's design, D4 to D8, D12 and D14.

## Context and problem statement

ADR 0006 serves only where a client reads no files, and 114 gives a host one serving process, at one fixed name and port with the repository in the path, which no checkout starts beside it (core.36). Status now reads every channel from the ledger, on any machine, and 132 asks that the same answer be a page: every repository the host knows together, one row each, and each repository whole at a path of its own, rendered on each request from the one core and from no timer. Where does the process answer, how is a repository named in its path, how does the process learn which repositories there are, who starts it, and when does it end?

## Considered options

For the server:

1. HTTP/1.1 over the standard library's sockets, one request per connection, `GET` alone, and the standard library's file lock.
2. An HTTP server crate, and a lock crate.

For who starts it:

1. The first asker that finds none, holding the host's one lock for as long as it runs and ending when idle; a host may also run it as a service of its own.
2. A service unit the release ships, which a person installs and starts.

For how it learns a repository:

1. A file of the host's, which a command run in a checkout writes, naming the repository's owner and name and the checkout.
2. Every checkout on the host found by a search of the disk.
3. The bindings file naming each repository.

## Decision outcome

The address is the core's, `127.0.0.1` port `7420`, and no setting, flag or variable moves it (114). A repository's path is `/<owner>/<name>/`, its owner and name the last two parts of the path of its landing's remote's address as git's configuration holds it, whatever host holds the remote, so the path names no forge and no one owner's store (49, core.1, core.18); a remote that ends in no owner and name leaves the repository unnamed, unreadable naming the remote and its address. The landing's remote is `[landing]`'s where the bindings file names one, and `origin` on a host with no file (118).

The server is option 1. Each request reads its head to the blank line and is answered with its length and the connection closed. `/serving.json` names the process, its revision and its id, so an asker tells it from anything else on the port; `/<owner>/<name>/status.json` is the status answer in the checkout that last named the repository, as `tailrace status --json` prints it there, and `/<owner>/<name>/status` its page; `/status.json` is every repository the host's file names, one row each, which the core makes from that checkout's status answer, and `/` its page (132, D14). A named repository answers 200 whatever its parts hold; one none named is absent, 404, saying no command run in a checkout of it on this host has named it; a file that cannot be read is unreadable, 500, with the parser's words (58, 59, core.19). Each page is one HTML document with no script, rendered from one answer of the core and nothing else, every string escaped and every refusal shown with the source's own words (57, core.31). Every request computes its answer anew, writes nothing and asks no store, signer or target (60, core.32); no row is kept between requests. Option 2 adds a dependency for one loopback route of `GET`s, which the bolt's units add none of.

Who starts it is option 1, on the user's word at 85bbf54, as sluice's one reader per host is (sluice 129). `tailrace status --page` names the checkout's repository, asks the address for `/serving.json`, and where nothing answers starts `tailrace serve` from its own binary, detached from its terminal and its signals, its working directory the state directory and never a checkout, and asks again until it answers; it prints the repository's own page's address, the root one link away. The process takes `serve.lock` in the state directory before it binds the address, so a second that finds it held says where the first answers and ends with 0, and two askers that both find none both find one process. It ends when nothing has asked it for ten minutes, letting the lock go. Nothing tells a person to start one; a host may run `tailrace serve` as a service of its own, and the lock keeps it one. Option 2 is rejected at 85bbf54: it asks every person to install and start a service before a page answers, and names a service manager the family does not.

How it learns a repository is option 1, ruled at dbe7ca0. The file is `repositories` in the state directory, `$XDG_STATE_HOME/tailrace`, else `~/.local/state/tailrace`, a TOML table per repository keyed by `owner/name` naming the checkout that last named it, readable with any file reader (62). Only `tailrace status --page` names a repository, since it is the asker that wants the page; plain `status` asks what tailrace holds and writes nothing (60, D12). The file is written under a lock of its own, by a temporary file renamed over it, and read on each request without a lock; an entry whose checkout is gone is its row's unreadable answer naming the path, beside every other row. Option 2 reads every disk on every request, or keeps what it found, which a timer would refresh; option 3 makes the operator list each repository by hand and moves a host setting with every clone.

### Consequences

- `tailrace status --page` answers on a host where nothing was installed beyond the binary, and every asker on the host is answered by one process at one address.
- The file of repositories is kept by the host and is not rebuilt from the store and git (63): the next `--page` in a checkout writes its entry again, and until then the process answers that repository absent, never a wrong answer.
- The root asks every named repository's ledger in turn on each request, a fetch each for a git ref ledger with a remote; a host with many repositories waits on them.
- The environments' part, and the four counts of each row, answer unreadable until a target is read (98), and versions' notes wait on the notes unit; the pages already show where each will go.
- Bolt 5's served layouts share the address and the process, and the suite cites 114 with the first of them.
