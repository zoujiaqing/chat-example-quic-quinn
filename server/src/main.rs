use anyhow::*;
use quinn::{Endpoint, Connection};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::result::Result::Ok;

extern crate protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let (cert_der, key_der) = protocol::common::generate_self_signed_cert()?;
    
    let server_config = protocol::common::configure_server(cert_der.clone(), key_der.clone_key())?;
    
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), protocol::common::SERVER_PORT);
    let endpoint = Endpoint::server(server_config, addr)?;
    println!("Listening on {}", endpoint.local_addr()?);
    
    // 保存证书到文件，以便客户端加载
    let _ = protocol::common::save_cert_and_key(&cert_der, &key_der);

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    println!("Connection established from: {}", connection.remote_address());
                    if let Err(e) = handle_connection(connection).await {
                        eprintln!("Connection error: {}", e);
                    }
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        });
    }

    Ok(())
}

async fn handle_connection(connection: Connection) -> Result<()> {
    while let Ok((mut send, mut recv)) = connection.accept_bi().await {
        let mut buf = Vec::new();
        while let Some(chunk) = recv.read_chunk(1024, false).await? {
            buf.extend_from_slice(&chunk.bytes);
        }
        println!("Received: {}", String::from_utf8_lossy(&buf));

        send.write_all(&buf).await?;
        send.finish();
    }
    Ok(())
}
