use tungstenite::connect;

fn main() {
    let (mut socket, response) = connect("ws://localhost:3012").expect("Can't connect");

    loop {
        let msg = socket.read().expect("Can't read message");
        println!("received message: {}", msg);
    }
    
    let _ = socket.close(None);
}