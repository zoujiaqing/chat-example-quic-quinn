use quinn::TransportConfig;
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use anyhow::*;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use quinn_proto::crypto::rustls::QuicClientConfig;

pub const SERVER_PORT: u16 = 5000;
pub const SERVER_CERT_PATH: &str = "cert.der";
pub const SERVER_KEY_PATH: &str = "key.der";

// 生成自签名证书
pub fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = CertificateDer::from(cert);
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(Vec::from(key_pair.serialized_der())));

    Ok((cert_der, key))
}

// 保存证书和密钥
pub fn save_cert_and_key(cert_der: &CertificateDer<'static>, key_der: &PrivateKeyDer<'static>) -> Result<()> {
    std::fs::write(SERVER_CERT_PATH, cert_der.as_ref())?;
    std::fs::write(SERVER_KEY_PATH, key_der.secret_der())?;
    Ok(())
}

// 加载证书和密钥
pub fn load_cert_and_key() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert_der = std::fs::read(SERVER_CERT_PATH)?;
    let key_der = std::fs::read(SERVER_KEY_PATH)?;
    let cert = CertificateDer::from(cert_der);
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
    Ok((cert, key))
}

// 配置服务器
pub fn configure_server(cert_der: CertificateDer<'static>, key_der: PrivateKeyDer<'static>) -> Result<quinn::ServerConfig> {
    let rustls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der.clone_key())
        .context("Failed to configure server with certificate and key")?;
    
    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(rustls_config)?));
    server_config.transport = Arc::new(TransportConfig::default());

    Ok(server_config)
}

// 配置客户端
pub fn configure_client(cert_der: CertificateDer<'static>) -> Result<quinn::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert_der).unwrap();

    let client_config = rustls::ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

    Ok(quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_config)?)))
}

// 获取服务器地址
pub fn get_server_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SERVER_PORT)
}
