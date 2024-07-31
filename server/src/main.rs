use std::{
    fs,
    sync::Arc,
    env,
    fs::File,
    io::{BufReader, Read},
    error::Error,
    path::Path,
};

use quinn_proto::crypto::rustls::QuicServerConfig;
use protocol::Message;
use quinn::{Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let server_config = if args.len() > 2 {
        // 如果提供了证书和密钥路径，则使用它们
        let cert_path = args[1].clone();
        let key_path = args[2].clone();
        configure_server_with_tls(cert_path, key_path)?
    } else {
        // 否则使用默认配置
        configure_server_without_tls()
    };

    // 创建一个 server 端点
    let endpoint = Endpoint::server(server_config, "[::1]:5000".parse()?)?;

    println!("Server listening on [::1]:5000");

    // 接受传入的连接
    while let Some(connecting) = endpoint.accept().await {
        tokio::spawn(async move {
            match connecting.await {
                Ok(new_conn) => {
                    if let Err(e) = handle_connection(new_conn).await {
                        eprintln!("Connection error: {:?}", e);
                    }
                },
                Err(e) => eprintln!("Connection error: {:?}", e),
            }
        });
    }

    // 让服务器继续运行
    endpoint.wait_idle().await;

    Ok(())
}

async fn handle_connection(conn: quinn::Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
        tokio::spawn(async move {
            if let Err(e) = handle_stream(&mut send, &mut recv).await {
                eprintln!("Stream error: {:?}", e);
            }
        });
    }
    Ok(())
}

async fn handle_stream(send: &mut quinn::SendStream, recv: &mut quinn::RecvStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut buffer = vec![0; 1024]; // 使用一个固定大小的缓冲区
    let n = recv.read(&mut buffer).await?.expect("Failed to read from stream");
    buffer.truncate(n); // 截断到实际读取的大小
    let message: Message = bincode::deserialize(&buffer)?;
    println!("Received: {}", message.content);

    let response = Message::new(&message.content);
    let response_bytes = bincode::serialize(&response)?;

    send.write_all(&response_bytes).await?;
    send.finish()?;

    Ok(())
}

fn configure_server_with_tls(cert_path: String, key_path: String) -> Result<ServerConfig, Box<dyn Error + Send + Sync>> {
    let certs = load_certs(&cert_path)?;
    let keys = load_keys(&key_path)?;

    let rustls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(keys[0].clone_key()))?;

    let server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(rustls_config)?));
    Ok(server_config)
}

fn configure_server_without_tls() -> ServerConfig {
    let rustls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![], PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(vec![]).clone_key())).unwrap(); // 临时占位符，实际应用中应提供证书和密钥

    let server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(rustls_config).unwrap()));
    server_config
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn Error + Send + Sync>> {
    let cert_chain = fs::read(path).context("failed to read certificate chain")?;
    let cert_chain = if Path::new(path).extension().map_or(false, |x| x == "der") {
        vec![CertificateDer::from(cert_chain)]
    } else {
        rustls_pemfile::certs(&mut &*cert_chain)
            .into_iter()
            .map(|cert| cert.map(CertificateDer::from))
            .collect::<Result<Vec<_>, _>>()
            .context("invalid PEM-encoded certificate")?
    };
    Ok(cert_chain)
}

fn load_keys(path: &str) -> Result<Vec<PrivatePkcs8KeyDer<'static>>, Box<dyn Error + Send + Sync>> {
    let mut keyfile = File::open(path)?;
    let mut buf = Vec::new();
    keyfile.read_to_end(&mut buf)?;

    let private_key = PrivatePkcs8KeyDer::from(buf);
    let keys = vec![private_key];
    Ok(keys)
}