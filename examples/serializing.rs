use resoxide_json::{Json, Token};
use resoxide_link::data_model::{Float3, Member, Slot};
use resoxide_link::messages::AddSlotData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slot = Slot::new(Slot::ROOT_SLOT_ID, "Slot".to_string())
        .with_tag("tag".to_string())
        .with_position(Float3::new(1.0, 2.0, 3.0));

    let message = resoxide_link::messages::Message::AddSlot(resoxide_link::messages::AddSlot {
        message_id: "MyID01".to_string(),
        data: AddSlotData::from(slot),
    });

    let json = message.to_token()?.serialize()?;
    println!("{}", json);

    let other_slot = Slot::from_token(&Token::deserialize_str(json.as_str())?)?;

    println!("{:?}", other_slot);

    let member = Member::from(Float3::new(45.0, -1.23, 0.4));

    let json = member.to_token()?.serialize()?;
    println!("{}", json);
    let other_member = Member::from_token(&Token::deserialize_str(json.as_str())?)?;
    println!("{:?}", other_member);
    
    Ok(())
}