use protocol::Message;

fn main() {
    let message = Message::new("Hello from client!");
    println!("Client: {}", message.content);
}
