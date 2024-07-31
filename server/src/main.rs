use protocol::Message;

fn main() {
    let message = Message::new("Hello from server!");
    println!("Server: {}", message.content);
}
