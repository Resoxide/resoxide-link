use std::thread::scope;
use serde::Serialize;
use tungstenite::connect;
use resoxide_link::data_model::Slot;
use resoxide_link::messages::{GetSlot, Message};
use resoxide_link::responses::Response;

fn prettify(bytes: &[u8]) -> String {
    let obj: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    serde_json::to_string_pretty(&obj).unwrap()
}

fn main() {

    scope(move |scope| {
        let (mut socket, response) = connect("ws://localhost:61587").expect("Can't connect");

        let message = Message::GetSlot(GetSlot {
            message_id: "MyID00".to_string(),
            slot_id: Slot::ROOT_SLOT_ID.to_string(),
            depth: -1,
            include_component_data: true,
        });

        socket.write(tungstenite::Message::Text(serde_json::to_string(&message).expect("Cannot encode message").into())).expect("Cannot send message");
        socket.flush().expect("Cannot flush socket");

        let msg = socket.read().expect("Can't read message");

        match msg {
            tungstenite::Message::Text(text) => {
                let response: Response = serde_json::from_slice(text.as_bytes()).expect("Cannot deserialize response");
                println!("decoded response: {:#?}", response);
            }
            _ => ()
        }

        let _ = socket.close(None);
    });
}