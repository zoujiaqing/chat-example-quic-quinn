use std::{
    net::SocketAddr,
    sync::Arc,
};

use quinn::{ClientConfig, Endpoint};
use quinn_proto::crypto::rustls::QuicClientConfig;
use protocol::Message;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 设置目标地址
    let server_addr: SocketAddr = "[::1]:5000".parse()?;

    // 配置客户端
    let client_config = configure_client_without_tls();

    let mut endpoint = Endpoint::client("[::]:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    // 连接到服务器
    let connection = endpoint.connect(server_addr, "localhost")?.await?;

    // 打开一个双向流
    let (mut send, mut recv) = connection.open_bi().await?;

    // 发送消息
    let message = Message::new("Hello, server!");
    let message_bytes = bincode::serialize(&message)?;
    send.write_all(&message_bytes).await?;
    send.finish();

    // 接收服务器的回显
    let mut buffer = vec![0; 1024];
    let n = recv.read(&mut buffer).await?.expect("Failed to read from stream");
    buffer.truncate(n);
    let received_message: Message = bincode::deserialize(&buffer)?;

    println!("Received: {}", received_message.content);

    Ok(())
}

fn configure_client_without_tls() -> ClientConfig {
    let mut root_store = rustls::RootCertStore::empty();
    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto).unwrap()))
}
