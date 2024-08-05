use anyhow::*;
use quinn::{Endpoint, Connection};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::result::Result::Ok;

extern crate protocol;

#[tokio::main]
async fn main() -> Result<()>
{
    let _ = rustls::crypto::ring::default_provider().install_default();

    let (cert_der, _) = protocol::common::load_cert_and_key()?;
    let client_config = protocol::common::configure_client(cert_der)?;

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    let server_addr = protocol::common::get_server_addr();
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    println!("Connected to {}", connection.remote_address());

    run_client(connection).await?;

    endpoint.wait_idle().await;
    Ok(())
}

async fn run_client(connection: Connection) -> Result<()> {
    loop {
        println!("Enter a message (or 'quit' to exit):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let message = input.trim();
        if message == "quit" {
            break;
        }

        let response = send_message(&connection, message).await?;
        println!("Server response: {}", response);
    }

    connection.close(0u32.into(), b"Done");
    Ok(())
}

async fn send_message(connection: &Connection, message: &str) -> Result<String> {
    let (mut send, mut recv) = connection.open_bi().await?;
    
    send.write_all(message.as_bytes()).await?;
    send.finish();

    let mut response = String::new();
    recv.read_to_string(&mut response).await?;

    Ok(response)
}
