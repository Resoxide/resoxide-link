use resoxide_link::data_model::Slot;
use resoxide_link::messages::{GetSlot, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = resoxide_link::client::Client::connect_port(16177).await?;
    let result = client.call(Message::GetSlot(GetSlot {
        message_id: Default::default(),
        depth: 0,
        include_component_data: false,
        slot_id: Slot::ROOT_SLOT_ID.to_string(),
    }), None).await?;
    println!("{:?}", result);
    client.close().await?;
    Ok(())
}
