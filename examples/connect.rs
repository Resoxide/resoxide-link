use std::fmt::format;
use std::thread::scope;
use tungstenite::{connect, Utf8Bytes};
use resoxide_link::data_model::Slot;
use resoxide_link::messages::{GetComponent, GetSlot, Message};
use resoxide_link::responses::Response;
use resoxide_json::{Json, Token};

fn message(msg: &Message) -> tungstenite::Message {
    tungstenite::Message::Text(
    msg.to_token().expect("Token").serialize().expect("Serialize").as_str().into()
    )
}

fn read_response(msg: &tungstenite::Message) -> Response {
    Response::from_token(&Token::deserialize_str(&msg.to_text().unwrap()).unwrap()).unwrap()
}

fn main() {
    let mut counter = 0;
    let mut get_id = move || {
        counter += 1;
        format!("MyID{counter:x}")
    };

    scope(move |scope| {
        let (mut socket, response) = connect("ws://localhost:38916").expect("Can't connect");

        let msg = Message::GetSlot(GetSlot {
            message_id: get_id(),
            slot_id: Slot::ROOT_SLOT_ID.to_string(),
            depth: 0,
            include_component_data: false,
        });

        let message_json = msg.to_token().expect("Can't serialize message")
            .serialize().expect("Can't serialize message");

        socket.write(tungstenite::Message::Text(message_json.as_str().into())).expect("Cannot send message");
        socket.flush().expect("Cannot flush socket");

        let msg = socket.read().expect("Can't read message");

        let mut components = vec![];

        match msg {
            tungstenite::Message::Text(text) => {
                println!("{}", text);
                let token = Token::deserialize_str(text.as_str()).expect("Can't deserialize json");
                //println!("{:#?}", token);
                let response = Response::from_token(&token).expect("Cannot deserialize response");
                println!("decoded response: {:#?}", response);
                let Response::SlotData(slot) = response else { panic!("Response isn't slot"); };
                for component in slot.data.components.expect("Components").iter() {
                    components.push(component.id.as_ref().expect("ID").clone());
                }
                for child in slot.data.children.as_ref().expect("Children").iter() {
                    if let Some(child_components) = &child.components {
                        for component in child_components.iter() {
                            components.push(component.id.as_ref().expect("ID").clone());
                        }
                    }
                }
            }
            _ => ()
        }

        for id in &components {
            socket.write(message(&Message::GetComponent(GetComponent { message_id: get_id(), component_id: id.clone() }))).expect("Can't send message");
            socket.flush();
            let r = socket.read().expect("Can't read response");
            println!("{:?}", r);
            let response = read_response(&r);
            println!("{:#?}", response);
        }

        let _ = socket.close(None);
    });
}