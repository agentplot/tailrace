//! A port on the loopback interface nothing listens on, for a test that
//! asks what a client is told there. A port a listener on port 0 was
//! handed and then dropped will not do: the host hands it again, to
//! another test's listener or to a connection's own end, and the asking
//! then reaches that.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};

/// A port nothing listens on: below the ranges every host hands a
/// listener on port 0 or a connection's own end from (32768 on Linux,
/// 49152 on macOS), and apart from the ports the tests start servers on,
/// so nothing takes it once chosen. The check binds it without
/// listening, so no asking is ever answered there, even while it is
/// checked.
pub fn unanswered() -> u16 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let n = NEXT.fetch_add(1, Ordering::Relaxed) + u64::from(std::process::id()) * 131;
        let port = 27000 + (n % 1000) as u16;
        let free = tokio::net::TcpSocket::new_v4()
            .and_then(|socket| socket.bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))));
        if free.is_ok() {
            return port;
        }
    }
}
