use resoxide_link::data_model::{Float3, Slot};
use resoxide_link::messages::AddSlotData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slot = Slot::new(Slot::ROOT_SLOT_ID, "Slot".to_string())
        .with_tag("tag".to_string())
        .with_position(Float3::new(1.0, 2.0, 3.0));

    let message = resoxide_link::messages::Message::AddSlot(resoxide_link::messages::AddSlot {
        message_id: "MyID01".to_string(),
        data: AddSlotData::from(slot),
    });

    let json = serde_json::to_string(&message)?;
    println!("{}", json);
    Ok(())
}