use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

static RELAYS: OnceLock<Mutex<HashMap<String, oneshot::Sender<()>>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, oneshot::Sender<()>>> {
    RELAYS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn stop_for_instance(instance_id: &str) {
    if let Some(tx) = registry().lock().unwrap().remove(instance_id) {
        let _ = tx.send(());
    }
}

/// Start a loopback SOCKS5 relay that accepts no-auth local connections from
/// Firefox and authenticates upstream with RFC 1929 username/password.
pub async fn start_for_instance(
    instance_id: &str,
    upstream_host: String,
    upstream_port: u16,
    username: String,
    password: String,
) -> Result<u16, String> {
    stop_for_instance(instance_id);
    let (addr, shutdown_tx) =
        spawn_relay(upstream_host, upstream_port, username, password).await?;
    registry()
        .lock()
        .unwrap()
        .insert(instance_id.to_string(), shutdown_tx);
    Ok(addr.port())
}

pub(crate) async fn spawn_relay(
    upstream_host: String,
    upstream_port: u16,
    username: String,
    password: String,
) -> Result<(SocketAddr, oneshot::Sender<()>), String> {
    if username.len() > 255 || password.len() > 255 {
        return Err("SOCKS5 username and password must each be at most 255 bytes".to_string());
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind local SOCKS5 relay: {}", e))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to read relay address: {}", e))?;

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    match accepted {
                        Ok((client, _)) => {
                            let host = upstream_host.clone();
                            let user = username.clone();
                            let pass = password.clone();
                            tokio::spawn(async move {
                                let _ = handle_client(client, &host, upstream_port, &user, &pass).await;
                            });
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });

    Ok((local_addr, shutdown_tx))
}

async fn handle_client(
    mut client: TcpStream,
    upstream_host: &str,
    upstream_port: u16,
    username: &str,
    password: &str,
) -> io::Result<()> {
    // Local Firefox greeting: [VER=5, NMETHODS, METHODS...]
    let mut head = [0u8; 2];
    client.read_exact(&mut head).await?;
    if head[0] != 0x05 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not SOCKS5"));
    }
    let mut methods = vec![0u8; head[1] as usize];
    client.read_exact(&mut methods).await?;

    if !methods.contains(&0x00) {
        client.write_all(&[0x05, 0xFF]).await?;
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "local client did not offer no-auth",
        ));
    }
    client.write_all(&[0x05, 0x00]).await?;

    // Connect upstream and authenticate with RFC 1929 username/password.
    let mut upstream = TcpStream::connect((upstream_host, upstream_port)).await?;
    upstream.write_all(&[0x05, 0x01, 0x02]).await?;

    let mut sel = [0u8; 2];
    upstream.read_exact(&mut sel).await?;
    if sel[0] != 0x05 || sel[1] != 0x02 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "upstream SOCKS5 server rejected username/password auth",
        ));
    }

    let u_bytes = username.as_bytes();
    let p_bytes = password.as_bytes();
    let mut auth_req = Vec::with_capacity(3 + u_bytes.len() + p_bytes.len());
    auth_req.push(0x01);
    auth_req.push(u_bytes.len() as u8);
    auth_req.extend_from_slice(u_bytes);
    auth_req.push(p_bytes.len() as u8);
    auth_req.extend_from_slice(p_bytes);
    upstream.write_all(&auth_req).await?;

    let mut auth_resp = [0u8; 2];
    upstream.read_exact(&mut auth_resp).await?;
    if auth_resp[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "upstream SOCKS5 authentication failed",
        ));
    }

    let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn relay_authenticates_upstream_and_forwards_bidirectional_traffic() {
        let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_addr = upstream_listener.local_addr().unwrap();

        let upstream_task = tokio::spawn(async move {
            let (mut sock, _) = upstream_listener.accept().await.unwrap();

            // Expect SOCKS5 greeting offering user/pass auth (0x02)
            let mut g = [0u8; 3];
            sock.read_exact(&mut g).await.unwrap();
            assert_eq!(g, [0x05, 0x01, 0x02]);
            sock.write_all(&[0x05, 0x02]).await.unwrap();

            // Expect RFC 1929 auth packet
            let mut v_ulen = [0u8; 2];
            sock.read_exact(&mut v_ulen).await.unwrap();
            assert_eq!(v_ulen[0], 0x01);
            let mut user = vec![0u8; v_ulen[1] as usize];
            sock.read_exact(&mut user).await.unwrap();
            let mut plen = [0u8; 1];
            sock.read_exact(&mut plen).await.unwrap();
            let mut pass = vec![0u8; plen[0] as usize];
            sock.read_exact(&mut pass).await.unwrap();
            assert_eq!(user, b"alice@org");
            assert_eq!(pass, b"p@ss:1#2");
            sock.write_all(&[0x01, 0x00]).await.unwrap();

            // Echo payload after auth
            let mut buf = [0u8; 5];
            sock.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"hello");
            sock.write_all(b"world").await.unwrap();
        });

        let (relay_addr, shutdown_tx) = spawn_relay(
            "127.0.0.1".to_string(),
            upstream_addr.port(),
            "alice@org".to_string(),
            "p@ss:1#2".to_string(),
        )
        .await
        .unwrap();

        let mut client = TcpStream::connect(relay_addr).await.unwrap();
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut resp = [0u8; 2];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(resp, [0x05, 0x00]);

        client.write_all(b"hello").await.unwrap();
        let mut reply = [0u8; 5];
        client.read_exact(&mut reply).await.unwrap();
        assert_eq!(&reply, b"world");

        let _ = shutdown_tx.send(());
        upstream_task.await.unwrap();
    }

    #[tokio::test]
    async fn relay_rejects_oversized_credentials() {
        let err = spawn_relay("127.0.0.1".into(), 1080, "a".repeat(256), "pw".into())
            .await
            .unwrap_err();
        assert!(err.contains("255"));
    }
}
