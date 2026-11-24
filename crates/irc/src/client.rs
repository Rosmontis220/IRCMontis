//! Async IRC engine: TLS connection, registration/SASL, auto-reconnect, event stream.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_rustls::TlsConnector;

use crate::parser::{self, Message};

pub const DEFAULT_HOST: &str = "irc.libera.chat";
pub const DEFAULT_PORT: u16 = 6697;
const SEND_CHUNK_BYTES: usize = 400;
const SEND_INTERVAL: Duration = Duration::from_millis(300);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaslAccount {
    pub account: String,
    pub password: String,
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}
fn default_port() -> u16 {
    DEFAULT_PORT
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectConfig {
    pub nick: String,
    #[serde(default)]
    pub sasl: Option<SaslAccount>,
    #[serde(default)]
    pub auto_join: Vec<String>,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Member {
    pub nick: String,
    pub prefix: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Event {
    Connected {
        nick: String,
    },
    Disconnected {
        reason: String,
        reconnect_in_secs: Option<u64>,
    },
    NickInUse {
        attempted: String,
    },
    SelfNick {
        nick: String,
    },
    Message {
        session: String,
        sender: String,
        text: String,
        kind: String,
        ts: i64,
        msgid: Option<String>,
    },
    Joined {
        channel: String,
        nick: String,
    },
    Parted {
        channel: String,
        nick: String,
        reason: Option<String>,
    },
    Kicked {
        channel: String,
        nick: String,
        setter: String,
        reason: Option<String>,
    },
    Quit {
        nick: String,
        reason: Option<String>,
    },
    NickChange {
        old: String,
        new: String,
    },
    Topic {
        channel: String,
        topic: String,
    },
    Names {
        channel: String,
        names: Vec<Member>,
    },
    Mode {
        channel: String,
        setter: String,
        modes: String,
        args: Vec<String>,
    },
    JoinFailed {
        channel: String,
        reason: String,
    },
    SendFailed {
        target: String,
        reason: String,
    },
    ServerNotice {
        text: String,
    },
}

pub enum Command {
    Connect(ConnectConfig),
    Disconnect,
    Send {
        target: String,
        text: String,
    },
    Join(String),
    Part {
        channel: String,
        reason: Option<String>,
    },
    SetNick(String),
}

#[derive(Clone)]
pub struct Handle {
    tx: mpsc::UnboundedSender<Command>,
}

impl Handle {
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }
}

pub fn spawn() -> (Handle, mpsc::UnboundedReceiver<Event>) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (ev_tx, ev_rx) = mpsc::unbounded_channel();
    tokio::spawn(run_loop(cmd_rx, ev_tx));
    (Handle { tx: cmd_tx }, ev_rx)
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("tcp: {0}")]
    Io(#[from] std::io::Error),
    #[error("tls: {0}")]
    Tls(#[from] rustls::Error),
    #[error("invalid server name")]
    Dns,
    #[error("connect timeout")]
    Timeout,
}

type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;
type ReadHalf = tokio::io::ReadHalf<TlsStream>;
type WriteHalf = tokio::io::WriteHalf<TlsStream>;

#[derive(Default)]
struct RegState {
    nick: String,
    registered: bool,
    caps: Vec<String>,
    cap_ls_done: bool,
    names_buf: HashMap<String, Vec<Member>>,
}

async fn connect(cfg: &ConnectConfig) -> Result<TlsStream, ConnectError> {
    let tcp = tokio::time::timeout(
        CONNECT_TIMEOUT,
        TcpStream::connect((cfg.host.as_str(), cfg.port)),
    )
    .await
    .map_err(|_| ConnectError::Timeout)??;
    tcp.set_nodelay(true)?;
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let name =
        rustls::pki_types::ServerName::try_from(cfg.host.clone()).map_err(|_| ConnectError::Dns)?;
    let stream = tokio::time::timeout(CONNECT_TIMEOUT, connector.connect(name, tcp))
        .await
        .map_err(|_| ConnectError::Timeout)??;
    Ok(stream)
}

async fn write_line(writer: &mut WriteHalf, line: &str) -> std::io::Result<()> {
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\r\n").await?;
    writer.flush().await
}

async fn send_opt(writer: &mut Option<WriteHalf>, line: &str) {
    if let Some(w) = writer.as_mut()
        && write_line(w, line).await.is_err()
    {
        *writer = None;
    }
}

fn backoff(attempt: u32) -> u64 {
    const TABLE: [u64; 6] = [1, 2, 4, 8, 16, 30];
    TABLE[(attempt.saturating_sub(1) as usize).min(TABLE.len() - 1)]
}

/// Channel session ids are ASCII-lowercase; nicks keep their case.
fn normalize_channel(name: &str) -> String {
    let trimmed = name.trim();
    let prefixed = if trimmed.starts_with('#') || trimmed.starts_with('&') {
        trimmed.to_string()
    } else {
        format!("#{trimmed}")
    };
    prefixed.to_ascii_lowercase()
}

fn is_channel(name: &str) -> bool {
    name.starts_with('#') || name.starts_with('&')
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn highest_prefix(token: &str) -> (String, String) {
    const ORDER: [char; 5] = ['~', '&', '@', '%', '+'];
    let mut prefix = String::new();
    let mut rest = token;
    for p in ORDER {
        if let Some(stripped) = rest.strip_prefix(p) {
            prefix.push(p);
            rest = stripped;
            break;
        }
    }
    (rest.to_string(), prefix)
}

async fn run_loop(mut cmd_rx: mpsc::UnboundedReceiver<Command>, ev: mpsc::UnboundedSender<Event>) {
    let mut config: Option<ConnectConfig> = None;
    let mut remembered: Vec<String> = Vec::new();
    let mut want_connected = false;
    let mut attempt: u32 = 0;
    let mut retry_at: Option<tokio::time::Instant> = None;
    let mut reader: Option<BufReader<ReadHalf>> = None;
    let mut writer: Option<WriteHalf> = None;
    let mut st = RegState::default();

    loop {
        if want_connected && reader.is_none() && retry_at.is_none() {
            let cfg = config.clone().expect("config set when want_connected");
            match connect(&cfg).await {
                Ok(stream) => {
                    let (r, w) = tokio::io::split(stream);
                    reader = Some(BufReader::new(r));
                    writer = Some(w);
                    st = RegState {
                        nick: cfg.nick.clone(),
                        ..RegState::default()
                    };
                    send_opt(&mut writer, "CAP LS 302").await;
                    send_opt(&mut writer, &format!("NICK {}", cfg.nick)).await;
                    send_opt(&mut writer, "USER ircmontis 0 * :IRCMontis").await;
                    attempt = 0;
                }
                Err(e) => {
                    attempt += 1;
                    let delay = backoff(attempt);
                    let _ = ev.send(Event::Disconnected {
                        reason: e.to_string(),
                        reconnect_in_secs: Some(delay),
                    });
                    retry_at = Some(tokio::time::Instant::now() + Duration::from_secs(delay));
                }
            }
        }

        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    None => break,
                    Some(Command::Connect(cfg)) => {
                        remembered = cfg.auto_join.iter().map(|c| normalize_channel(c)).collect();
                        remembered.dedup();
                        config = Some(cfg);
                        want_connected = true;
                        attempt = 0;
                        retry_at = None;
                        reader = None;
                        writer = None;
                    }
                    Some(Command::Disconnect) => {
                        want_connected = false;
                        retry_at = None;
                        reader = None;
                        writer = None;
                        let _ = ev.send(Event::Disconnected { reason: "manual".to_string(), reconnect_in_secs: None });
                    }
                    Some(Command::Send { target, text }) => {
                        let ready = st.registered && writer.is_some();
                        if ready {
                            for chunk in parser::split_message(&text, SEND_CHUNK_BYTES) {
                                send_opt(&mut writer, &format!("PRIVMSG {target} :{chunk}")).await;
                                if writer.is_none() { break; }
                                tokio::time::sleep(SEND_INTERVAL).await;
                            }
                        } else {
                            let _ = ev.send(Event::SendFailed { target, reason: "not connected".to_string() });
                        }
                    }
                    Some(Command::Join(ch)) => {
                        let ch = normalize_channel(&ch);
                        if ch.len() > 1 && !remembered.iter().any(|c| c == &ch) {
                            remembered.push(ch.clone());
                        }
                        if st.registered {
                            send_opt(&mut writer, &format!("JOIN {ch}")).await;
                        }
                    }
                    Some(Command::Part { channel, reason }) => {
                        remembered.retain(|c| c != &channel);
                        let reason = reason.unwrap_or_else(|| "IRCMontis".to_string());
                        send_opt(&mut writer, &format!("PART {channel} :{reason}")).await;
                    }
                    Some(Command::SetNick(n)) => {
                        st.nick = n.clone();
                        send_opt(&mut writer, &format!("NICK {n}")).await;
                    }
                }
            }
            line = async {
                match reader.as_mut() {
                    Some(r) => {
                        let mut buf = String::new();
                        match r.read_line(&mut buf).await {
                            Ok(0) => Ok(None),
                            Ok(_) => Ok(Some(buf.trim_end_matches(['\r', '\n']).to_string())),
                            Err(e) => Err(e),
                        }
                    }
                    None => std::future::pending::<std::io::Result<Option<String>>>().await,
                }
            } => {
                match line {
                    Ok(Some(line)) => {
                        if let Some(m) = parser::parse(&line) {
                            let alive = handle_message(&mut writer, &mut st, &m, &ev, &mut remembered, config.as_ref()).await;
                            if !alive {
                                reader = None;
                                writer = None;
                                if want_connected {
                                    attempt += 1;
                                    let delay = backoff(attempt);
                                    let _ = ev.send(Event::Disconnected { reason: "server closed connection".to_string(), reconnect_in_secs: Some(delay) });
                                    retry_at = Some(tokio::time::Instant::now() + Duration::from_secs(delay));
                                }
                            }
                        }
                    }
                    Ok(None) | Err(_) => {
                        reader = None;
                        writer = None;
                        if want_connected {
                            attempt += 1;
                            let delay = backoff(attempt);
                            let _ = ev.send(Event::Disconnected { reason: "connection lost".to_string(), reconnect_in_secs: Some(delay) });
                            retry_at = Some(tokio::time::Instant::now() + Duration::from_secs(delay));
                        }
                    }
                }
            }
            _ = async {
                match retry_at {
                    Some(t) => tokio::time::sleep_until(t).await,
                    None => std::future::pending::<()>().await,
                }
            } => {
                retry_at = None;
            }
        }
    }
}

/// Returns false when the connection must be dropped (ERROR line).
async fn handle_message(
    writer: &mut Option<WriteHalf>,
    st: &mut RegState,
    m: &Message,
    ev: &mpsc::UnboundedSender<Event>,
    remembered: &mut Vec<String>,
    config: Option<&ConnectConfig>,
) -> bool {
    let sender = m
        .prefix
        .as_ref()
        .map(|p| p.nick().to_string())
        .unwrap_or_default();
    let is_server = m
        .prefix
        .as_ref()
        .map(|p| !p.raw.contains('!'))
        .unwrap_or(true);
    let trailing = m.trailing().unwrap_or("").to_string();
    let p = &m.params;

    match m.command.as_str() {
        "PING" => {
            send_opt(
                writer,
                &format!("PONG :{}", p.first().map(|s| s.as_str()).unwrap_or("")),
            )
            .await;
        }
        "CAP" => {
            let sub = p.get(1).map(|s| s.as_str()).unwrap_or("");
            match sub {
                "LS" => {
                    let continuation = p.get(2).map(|s| s.as_str()) == Some("*");
                    // cap list arrives as the trailing; params only hold the continuation marker
                    let caps = if trailing.is_empty() {
                        p.last().map(|s| s.as_str()).unwrap_or("")
                    } else {
                        trailing.as_str()
                    };
                    st.caps
                        .extend(caps.split_whitespace().map(|s| s.to_string()));
                    if !continuation {
                        st.cap_ls_done = true;
                        let wants_sasl = config.and_then(|c| c.sasl.as_ref()).is_some();
                        // servers may advertise sasl with a mechanism list: sasl=PLAIN,...
                        if wants_sasl
                            && st
                                .caps
                                .iter()
                                .any(|c| c == "sasl" || c.starts_with("sasl="))
                        {
                            send_opt(writer, "CAP REQ :sasl").await;
                        } else {
                            send_opt(writer, "CAP END").await;
                        }
                    }
                }
                "ACK" => {
                    if trailing.contains("sasl") {
                        send_opt(writer, "AUTHENTICATE PLAIN").await;
                    } else {
                        send_opt(writer, "CAP END").await;
                    }
                }
                "NAK" => {
                    let _ = ev.send(Event::ServerNotice {
                        text: "server refused capability request".to_string(),
                    });
                    send_opt(writer, "CAP END").await;
                }
                _ => {}
            }
        }
        "AUTHENTICATE" => {
            if p.first().map(|s| s.as_str()) == Some("+") {
                if let Some(sasl) = config.and_then(|c| c.sasl.as_ref()) {
                    let payload = format!("{}\0{}\0{}", sasl.account, sasl.account, sasl.password);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
                    send_opt(writer, &format!("AUTHENTICATE {encoded}")).await;
                } else {
                    send_opt(writer, "AUTHENTICATE *").await;
                }
            }
        }
        "001" => {
            st.registered = true;
            if let Some(nick) = p.first() {
                st.nick = nick.clone();
            }
            let _ = ev.send(Event::Connected {
                nick: st.nick.clone(),
            });
            for ch in remembered.clone() {
                send_opt(writer, &format!("JOIN {ch}")).await;
            }
        }
        "433" | "436" => {
            if st.registered {
                let _ = ev.send(Event::ServerNotice { text: trailing });
            } else {
                let _ = ev.send(Event::NickInUse {
                    attempted: st.nick.clone(),
                });
            }
        }
        "432" => {
            let _ = ev.send(Event::ServerNotice { text: trailing });
        }
        "903" => {
            send_opt(writer, "CAP END").await;
        }
        "904" | "905" | "906" | "907" => {
            let _ = ev.send(Event::ServerNotice {
                text: format!("SASL: {trailing}"),
            });
            send_opt(writer, "CAP END").await;
        }
        "PRIVMSG" | "NOTICE" => {
            let is_notice = m.command == "NOTICE";
            let target = p.first().cloned().unwrap_or_default();
            let text = p.get(1).cloned().unwrap_or_default();
            if is_notice && (!st.registered || is_server) {
                let _ = ev.send(Event::ServerNotice {
                    text: parser::plain_text(&text),
                });
                return true;
            }
            if let Some(action) = parser::ctcp_action(&text) {
                let session = if is_channel(&target) {
                    target.clone()
                } else {
                    sender.clone()
                };
                let _ = ev.send(Event::Message {
                    session,
                    sender: sender.clone(),
                    text: action.to_string(),
                    kind: "action".to_string(),
                    ts: now_millis(),
                    msgid: m.tag("msgid").map(|s| s.to_string()),
                });
                return true;
            }
            if text.starts_with('\x01') {
                // CTCP query: answer VERSION/PING, ignore the rest. Never reply to NOTICE.
                if !is_notice {
                    let inner = text.trim_matches('\x01');
                    let (cmd, rest) = inner.split_once(' ').unwrap_or((inner, ""));
                    let reply = match cmd {
                        "VERSION" => Some("VERSION IRCMontis 0.1.0".to_string()),
                        "PING" => Some(format!("PING {rest}")),
                        _ => None,
                    };
                    if let Some(r) = reply {
                        send_opt(writer, &format!("NOTICE {sender} :\x01{r}\x01")).await;
                    }
                }
                return true;
            }
            let session = if is_channel(&target) {
                normalize_channel(&target)
            } else {
                sender.clone()
            };
            let _ = ev.send(Event::Message {
                session,
                sender: sender.clone(),
                text,
                kind: if is_notice {
                    "notice".to_string()
                } else {
                    "chat".to_string()
                },
                ts: now_millis(),
                msgid: m.tag("msgid").map(|s| s.to_string()),
            });
        }
        "JOIN" => {
            let channel =
                normalize_channel(p.first().map(|s| s.as_str()).unwrap_or(trailing.as_str()));
            if sender.eq_ignore_ascii_case(&st.nick) && !remembered.iter().any(|c| c == &channel) {
                remembered.push(channel.clone());
            }
            let _ = ev.send(Event::Joined {
                channel,
                nick: sender.clone(),
            });
        }
        "PART" => {
            let channel = normalize_channel(p.first().map(|s| s.as_str()).unwrap_or(""));
            if sender.eq_ignore_ascii_case(&st.nick) {
                remembered.retain(|c| c != &channel);
            }
            let _ = ev.send(Event::Parted {
                channel,
                nick: sender.clone(),
                reason: if trailing.is_empty() {
                    None
                } else {
                    Some(trailing.clone())
                },
            });
        }
        "KICK" => {
            let channel = normalize_channel(p.first().map(|s| s.as_str()).unwrap_or(""));
            let nick = p.get(1).cloned().unwrap_or_default();
            if nick.eq_ignore_ascii_case(&st.nick) {
                remembered.retain(|c| c != &channel);
            }
            let _ = ev.send(Event::Kicked {
                channel,
                nick,
                setter: sender.clone(),
                reason: if trailing.is_empty() {
                    None
                } else {
                    Some(trailing.clone())
                },
            });
        }
        "QUIT" => {
            let _ = ev.send(Event::Quit {
                nick: sender.clone(),
                reason: if trailing.is_empty() {
                    None
                } else {
                    Some(trailing.clone())
                },
            });
        }
        "NICK" => {
            let new = p.first().cloned().unwrap_or(trailing.clone());
            if sender.eq_ignore_ascii_case(&st.nick) {
                st.nick = new.clone();
                let _ = ev.send(Event::SelfNick { nick: new.clone() });
            }
            let _ = ev.send(Event::NickChange {
                old: sender.clone(),
                new,
            });
        }
        "TOPIC" => {
            let _ = ev.send(Event::Topic {
                channel: normalize_channel(p.first().map(|s| s.as_str()).unwrap_or("")),
                topic: trailing.clone(),
            });
        }
        "332" => {
            let _ = ev.send(Event::Topic {
                channel: normalize_channel(p.get(1).map(|s| s.as_str()).unwrap_or("")),
                topic: trailing.clone(),
            });
        }
        "331" => {
            let _ = ev.send(Event::Topic {
                channel: normalize_channel(p.get(1).map(|s| s.as_str()).unwrap_or("")),
                topic: String::new(),
            });
        }
        "353" => {
            let channel = normalize_channel(p.get(2).map(|s| s.as_str()).unwrap_or(""));
            let entry = st.names_buf.entry(channel).or_default();
            for token in trailing.split_whitespace() {
                let (nick, prefix) = highest_prefix(token);
                if !nick.is_empty() {
                    entry.push(Member { nick, prefix });
                }
            }
        }
        "366" => {
            let channel = normalize_channel(p.get(1).map(|s| s.as_str()).unwrap_or(""));
            if let Some(names) = st.names_buf.remove(&channel) {
                let _ = ev.send(Event::Names { channel, names });
            }
        }
        "MODE" => {
            let target = p.first().cloned().unwrap_or_default();
            if is_channel(&target) {
                let _ = ev.send(Event::Mode {
                    channel: normalize_channel(&target),
                    setter: sender.clone(),
                    modes: p.get(1).cloned().unwrap_or_default(),
                    args: p.iter().skip(2).cloned().collect(),
                });
            }
        }
        "471" | "473" | "474" | "475" | "477" => {
            let _ = ev.send(Event::JoinFailed {
                channel: normalize_channel(p.get(1).map(|s| s.as_str()).unwrap_or("")),
                reason: trailing.clone(),
            });
        }
        "401" | "404" => {
            let _ = ev.send(Event::SendFailed {
                target: p.get(1).cloned().unwrap_or_default(),
                reason: trailing.clone(),
            });
        }
        "ERROR" => {
            let _ = ev.send(Event::ServerNotice {
                text: trailing.clone(),
            });
            return false;
        }
        _ => {}
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_table_caps_at_30() {
        assert_eq!(backoff(1), 1);
        assert_eq!(backoff(3), 4);
        assert_eq!(backoff(6), 30);
        assert_eq!(backoff(99), 30);
    }

    #[test]
    fn channel_normalization() {
        assert_eq!(normalize_channel("#Foo"), "#foo");
        assert_eq!(normalize_channel("bar"), "#bar");
        assert_eq!(normalize_channel("  #Baz  "), "#baz");
        assert!(is_channel("#a"));
        assert!(!is_channel("nick"));
    }

    #[test]
    fn member_prefix_extraction() {
        assert_eq!(highest_prefix("@op"), ("op".to_string(), "@".to_string()));
        assert_eq!(highest_prefix("+v"), ("v".to_string(), "+".to_string()));
        assert_eq!(
            highest_prefix("plain"),
            ("plain".to_string(), String::new())
        );
    }

    // regression: select! evaluates disabled-branch expressions, so the retry
    // branch must not unwrap a None retry_at while idle
    #[tokio::test]
    async fn loop_survives_idle_and_manual_disconnect() {
        let (handle, mut ev) = spawn();
        handle.send(Command::Disconnect);
        let got = tokio::time::timeout(Duration::from_secs(2), ev.recv())
            .await
            .expect("event timeout")
            .expect("channel closed");
        assert!(matches!(got, Event::Disconnected { reason, .. } if reason == "manual"));
    }

    // regression: CAP LS cap list is the trailing, not a middle param
    #[tokio::test]
    async fn cap_ls_accumulates_trailing_caps() {
        let mut st = RegState::default();
        let (ev, _rx) = mpsc::unbounded_channel();
        let mut remembered = Vec::new();
        let mut writer = None;
        let cfg = ConnectConfig {
            nick: "t".to_string(),
            sasl: Some(SaslAccount {
                account: "t".to_string(),
                password: "p".to_string(),
            }),
            auto_join: vec![],
            host: "h".to_string(),
            port: 6697,
        };
        let m1 = parser::parse(":s CAP * LS * :account-notify").unwrap();
        handle_message(&mut writer, &mut st, &m1, &ev, &mut remembered, Some(&cfg)).await;
        assert!(!st.cap_ls_done);
        let m2 =
            parser::parse(":s CAP * LS :sasl=ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN").unwrap();
        handle_message(&mut writer, &mut st, &m2, &ev, &mut remembered, Some(&cfg)).await;
        assert!(st.cap_ls_done);
        assert!(
            st.caps
                .iter()
                .any(|c| c == "sasl" || c.starts_with("sasl="))
        );
    }
}
