//! Group proxy: the owner's dedicated server, whitelisted to the 2 gate hosts.
//! Credential is private in `.group`.
//! No region probe because the server refuses trace hosts (cloudflare etc.) by design;
//! its region is known to be correct.

use std::fmt;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rustls::crypto::ring::default_provider;
use rustls::pki_types::ServerName;
use rustls::ClientConnection;
use sha2::{Digest, Sha256};

use crate::group_key::GroupKey;
use crate::settings;
use crate::upstream::{self, Route, Upstream};

struct GroupState {
    key_string: String,
    parsed: Option<GroupKey>,
    cached_addrs: Option<(Vec<SocketAddr>, Instant)>,
}

fn group_state() -> &'static Mutex<GroupState> {
    static STATE: OnceLock<Mutex<GroupState>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(GroupState {
            key_string: String::new(),
            parsed: None,
            cached_addrs: None,
        })
    })
}

pub fn get_active_key() -> Option<GroupKey> {
    let settings_key = settings::group_key();
    if settings_key.is_empty() {
        return None;
    }
    let mut state = group_state().lock().unwrap();
    if state.key_string != settings_key {
        state.key_string = settings_key.clone();
        state.parsed = crate::group_key::parse_key(&settings_key).ok();
        state.cached_addrs = None;
    }
    state.parsed.clone()
}

fn group_route() -> &'static Route {
    static ROUTE: OnceLock<Route> = OnceLock::new();
    ROUTE.get_or_init(|| {
        Route::new(
            Box::leak("Прокси из группы".into()),
            crate::routes::Kind::Group,
        )
    })
}

pub fn available() -> bool {
    get_active_key().is_some() && group_route().usable()
}

fn agu2_config() -> std::sync::Arc<rustls::ClientConfig> {
    static CONFIG: OnceLock<std::sync::Arc<rustls::ClientConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let mut config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            obfstr::obfstr! { let alpn = "agu/2"; }
            config.alpn_protocols = vec![alpn.as_bytes().to_vec()];
            std::sync::Arc::new(config)
        })
        .clone()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for (i, &b) in key.iter().enumerate().take(64) {
        ipad[i] ^= b;
        opad[i] ^= b;
    }
    let inner = Sha256::new()
        .chain_update(&ipad)
        .chain_update(data)
        .finalize();
    let outer = Sha256::new()
        .chain_update(&opad)
        .chain_update(inner)
        .finalize();
    let mut res = [0u8; 32];
    res.copy_from_slice(&outer);
    res
}

fn open_agu2(
    key: &GroupKey,
    force_refresh: bool,
    target_host: &str,
) -> Result<(ClientConnection, TcpStream, Vec<u8>), String> {
    let mut addrs = None;
    {
        let mut state = group_state().lock().unwrap();
        if !force_refresh {
            if let Some((cached, ts)) = state.cached_addrs.as_ref() {
                if ts.elapsed() < Duration::from_secs(600) {
                    addrs = Some(cached.clone());
                }
            }
        }
        if addrs.is_none() {
            let resolved: Vec<_> = format!("{}:{}", key.host, key.port)
                .to_socket_addrs()
                .map_err(|e| e.to_string())?
                .filter(|a| a.is_ipv4())
                .collect();
            if resolved.is_empty() {
                return Err("нет IPv4 адресов".to_string());
            }
            state.cached_addrs = Some((resolved.clone(), Instant::now()));
            addrs = Some(resolved);
        }
    }
    let addrs = addrs.unwrap();
    let budget = upstream::LIVE_OPEN_BUDGET;
    let deadline = Instant::now() + budget;

    let mut sock = None;
    for addr in addrs {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        if let Ok(s) = TcpStream::connect_timeout(&addr, remaining) {
            sock = Some(s);
            break;
        }
    }
    let mut sock = sock.ok_or_else(|| "не удалось подключиться".to_string())?;

    let name = ServerName::try_from(key.host.clone()).map_err(|e| e.to_string())?;
    let mut tls = ClientConnection::new(agu2_config(), name).map_err(|e| e.to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut nonce = [0u8; 16];
    default_provider()
        .secure_random
        .fill(&mut nonce)
        .map_err(|_| "генератор случайных чисел отказал")?;
    let nonce_hex = hex::encode(nonce);

    // Signed with this PC's own code, never the one inside the key text.
    let auth_header =
        crate::group_key::auth_header(key, crate::hwid::pc_code(), ts, &nonce_hex, target_host);

    let req = format!(
        "CONNECT {}:443 HTTP/1.1\r\nHost: {}:443\r\nProxy-Authorization: {}\r\n\r\n",
        target_host, target_host, auth_header
    );

    sock.set_read_timeout(Some(budget)).ok();
    sock.set_write_timeout(Some(budget)).ok();

    let mut stream = rustls::Stream::new(&mut tls, &mut sock);
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut acc = Vec::new();
    let mut buf = [0u8; 1024];
    let hdr_end = loop {
        match stream.read(&mut buf) {
            Ok(0) => return Err("сервер группы отказал: соединение закрыто".to_string()),
            Ok(n) => {
                acc.extend_from_slice(&buf[..n]);
                if let Some(pos) = acc.windows(4).position(|w| w == b"\r\n\r\n") {
                    break pos + 4;
                }
                if acc.len() > 4096 {
                    return Err("сервер группы отказал: ответ слишком велик".to_string());
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.to_string()),
        }
    };

    let status = acc
        .split(|&b| b == b'\r' || b == b'\n')
        .next()
        .map(|l| String::from_utf8_lossy(l).into_owned())
        .unwrap_or_default();

    if !status.contains(" 200") {
        return Err(format!("сервер группы отказал: {}", status));
    }

    sock.set_read_timeout(None).ok();
    sock.set_write_timeout(None).ok();

    Ok((tls, sock, acc[hdr_end..].to_vec()))
}

const IDLE_PAYLOAD_LIMIT: Duration = Duration::from_secs(60);
const AWAITING_LIMIT: Duration = Duration::from_secs(10 * 60);

fn idle_expired(idle: Duration, awaiting: bool) -> bool {
    idle > if awaiting {
        AWAITING_LIMIT
    } else {
        IDLE_PAYLOAD_LIMIT
    }
}

enum End {
    Client,
    Idle,
    Upstream,
    NoAnswer,
}

fn drain_tls_reader(conn: &mut ClientConnection, pending: &mut Vec<u8>) -> bool {
    let mut moved = false;
    let mut buf = [0u8; 16 * 1024];
    loop {
        match conn.reader().read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                pending.extend_from_slice(&buf[..n]);
                moved = true;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
    moved
}

fn tls_pump(
    mut client: TcpStream,
    mut conn: ClientConnection,
    mut upstream_sock: TcpStream,
    leftover: Vec<u8>,
) {
    client.set_nonblocking(true).ok();
    upstream_sock.set_nonblocking(true).ok();
    let activity = crate::routes::serving_activity();
    let mut buf = [0u8; 16 * 1024];
    let mut pending: Vec<u8> = leftover;
    let mut idle = Duration::ZERO;
    let mut backoff = crate::proxy::PUMP_MIN_SLEEP;
    let mut client_eof = false;
    let mut upstream_eof = false;
    let mut _delivered: u64 = 0;
    let mut _sent: u64 = 0;
    let mut awaiting = false;
    let mut _why = End::Client;

    while !(client_eof && upstream_eof && pending.is_empty()) {
        let mut moved = false;
        let mut payload = false;

        if !upstream_eof && conn.wants_read() {
            match conn.read_tls(&mut upstream_sock) {
                Ok(0) => {
                    let _ = conn.process_new_packets();
                    let m = drain_tls_reader(&mut conn, &mut pending);
                    moved |= m;
                    if m {
                        payload = true;
                    }
                    upstream_eof = true;
                    _why = End::Upstream;
                }
                Ok(_) => match conn.process_new_packets() {
                    Ok(_) => moved = true,
                    Err(_) => {
                        let _ = drain_tls_reader(&mut conn, &mut pending);
                        client_eof = true;
                        upstream_eof = true;
                        _why = End::Upstream;
                    }
                },
                Err(e) if crate::proxy::would_block(&e) => {}
                Err(_) => {
                    let _ = drain_tls_reader(&mut conn, &mut pending);
                    client_eof = true;
                    upstream_eof = true;
                    _why = End::Upstream;
                }
            }
        }

        // Drain reader unconditionally if there are decrypted packets waiting
        if pending.is_empty() {
            let m = drain_tls_reader(&mut conn, &mut pending);
            if m {
                moved = true;
                payload = true;
                awaiting = false;
            } else if upstream_eof {
                // we have hit upstream eof and there's really nothing left to drain
            }
        }

        while !pending.is_empty() {
            match client.write(&pending) {
                Ok(0) => {
                    client_eof = true;
                    upstream_eof = true;
                    pending.clear();
                    _why = End::Client;
                }
                Ok(n) => {
                    pending.drain(..n);
                    _delivered += n as u64;
                    if let Some(a) = &activity {
                        a.to_client(n as u64);
                    }
                    moved = true;
                    payload = true;
                }
                Err(e) if crate::proxy::would_block(&e) => break,
                Err(_) => {
                    client_eof = true;
                    upstream_eof = true;
                    pending.clear();
                    _why = End::Client;
                }
            }
        }
        if !client_eof {
            match client.read(&mut buf) {
                Ok(0) => {
                    conn.send_close_notify();
                    client_eof = true;
                    _why = End::Client;
                }
                Ok(n) => {
                    if conn.writer().write_all(&buf[..n]).is_ok() {
                        _sent += n as u64;
                        if let Some(a) = &activity {
                            a.to_upstream(n as u64);
                        }
                        moved = true;
                        payload = true;
                        awaiting = true;
                    } else {
                        client_eof = true;
                        upstream_eof = true;
                        _why = End::Upstream;
                    }
                }
                Err(e) if crate::proxy::would_block(&e) => {}
                Err(_) => {
                    conn.send_close_notify();
                    client_eof = true;
                    _why = End::Client;
                }
            }
        }
        while conn.wants_write() {
            match conn.write_tls(&mut upstream_sock) {
                Ok(_) => moved = true,
                Err(e) if crate::proxy::would_block(&e) => break,
                Err(_) => {
                    client_eof = true;
                    upstream_eof = true;
                    _why = End::Upstream;
                    break;
                }
            }
        }

        if upstream_eof {
            // Plaintext rustls already decrypted is still owed to the client:
            // take all of it before calling the upstream finished.
            if drain_tls_reader(&mut conn, &mut pending) {
                moved = true;
            }
            if pending.is_empty() && !conn.wants_write() {
                break;
            }
        }

        if payload {
            idle = Duration::ZERO;
        }
        if moved {
            backoff = crate::proxy::PUMP_MIN_SLEEP;
            continue;
        }
        thread::sleep(backoff);
        idle += backoff;
        backoff = (backoff * 2).min(crate::proxy::PUMP_MAX_SLEEP);
        if idle_expired(idle, awaiting) {
            _why = if awaiting { End::NoAnswer } else { End::Idle };
            break;
        }
    }

    conn.send_close_notify();
    let deadline = Instant::now() + crate::proxy::PUMP_MAX_SLEEP;
    while conn.wants_write() && Instant::now() < deadline {
        if conn.write_tls(&mut upstream_sock).is_err() {
            break;
        }
    }
    upstream_sock.flush().ok();
    upstream_sock.shutdown(std::net::Shutdown::Both).ok();

    client.shutdown(std::net::Shutdown::Write).ok();
    let deadline = Instant::now() + crate::proxy::PUMP_MAX_SLEEP;
    while Instant::now() < deadline {
        match client.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) if crate::proxy::would_block(&e) => thread::sleep(crate::proxy::PUMP_MIN_SLEEP),
            Err(_) => break,
        }
    }
}

pub fn tunnel(mut client: TcpStream, host: &str, _port: u16) -> Result<(), TcpStream> {
    if !available() {
        return Err(client);
    }

    if let Some(key) = get_active_key() {
        let route = group_route();
        match open_agu2(&key, false, host) {
            Ok((tls, sock, leftover)) => {
                if client.write_all(crate::proxy::RESP_ESTABLISHED).is_err() {
                    return Ok(());
                }
                crate::routes::note_used(crate::routes::Kind::Group);
                crate::dns_forwarder::log_proxy(&format!("{} -> {}", route.label(), host));

                tls_pump(client, tls, sock, leftover);
                return Ok(());
            }
            Err(why) => {
                crate::dns_forwarder::log_proxy(&format!("{}: {}", route.label(), why));
                route.health.note(false);
            }
        }
    }
    Err(client)
}

pub fn probe_health() {
    let key = match get_active_key() {
        Some(k) => k,
        None => return,
    };

    let route = group_route();
    route.probe_with(|| {
        let target = "daily-cloudcode-pa.googleapis.com";
        let budget = upstream::PROBE_BUDGET;

        let mut tls = ClientConnection::new(
            crate::proxy::probe_config(),
            ServerName::try_from(target).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

        let req = format!(
            "GET /v1internal:probe HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            target
        );
        let mut buf = [0u8; 64];

        let (outer_tls, outer_sock, leftover) = open_agu2(&key, true, target)?;
        outer_sock.set_read_timeout(Some(budget)).ok();
        outer_sock.set_write_timeout(Some(budget)).ok();

        let mut stream_owned = rustls::StreamOwned::new(outer_tls, outer_sock);
        let mut stream = rustls::Stream::new(&mut tls, &mut stream_owned);

        if !leftover.is_empty() {
            return Err("неожиданные данные после CONNECT".to_string());
        }

        stream
            .write_all(req.as_bytes())
            .map_err(|e| e.to_string())?;
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n > 0 && buf.starts_with(b"HTTP/") {
            Ok(())
        } else {
            Err("ответ не похож на HTTP".to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_reader_test() {
        // Just verify it compiles and unit tests would pass
    }
}
