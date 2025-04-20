use serde::{
    de::{self, Unexpected}, Deserialize,
};
use serde_json::Value;

trait MessageType {
    const TYPE: u64;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompletionMessage {
    pub invocation_id: String,
    pub result: Value,
    pub error: Option<String>,
}

impl MessageType for CompletionMessage {
    const TYPE: u64 = 3;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InvocationMessage {
    pub target: String,
    pub arguments: Vec<String>,
}

impl MessageType for InvocationMessage {
    const TYPE: u64 = 1;
}

// enum MessageType {
//     Invocation = 1,
//     StreamItem = 2,
//     Completion = 3,
//     StreamInvocation = 4,
//     CancelInvocation = 5,
//     Ping = 6,
//     Close = 7,
// }

#[derive(Debug)]
pub enum SignalRMessage {
    Ping,
    Invocation(InvocationMessage),
    Completion(CompletionMessage),
}

// Messages are  _almost_ an internally tagged enum, except Serde
// currently only supports the tag being the literal name of the enum,
// not a number like SignalR sends.
impl<'de> Deserialize<'de> for SignalRMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;

        match value.get("type").and_then(Value::as_u64) {
            Some(InvocationMessage::TYPE) => {
                let inner_message = InvocationMessage::deserialize(value).map_err(|_| {
                    de::Error::invalid_type(Unexpected::StructVariant, &"a CompletionMessage")
                })?;

                Ok(SignalRMessage::Invocation(inner_message))
            }
            Some(2) => unimplemented!("can't deserialize StreamItem"),
            Some(CompletionMessage::TYPE) => {
                let inner_message = CompletionMessage::deserialize(value).map_err(|_| {
                    de::Error::invalid_type(Unexpected::StructVariant, &"a CompletionMessage")
                })?;

                Ok(SignalRMessage::Completion(inner_message))
            }
            Some(4) => unimplemented!("can't deserialize StreamInvocation"),
            Some(5) => unimplemented!("can't deserialize CancelInvocation"),
            Some(6) => Ok(SignalRMessage::Ping),
            Some(7) => unimplemented!("can't deserialize Close"),
            Some(num) => Err(de::Error::invalid_value(
                Unexpected::Unsigned(num),
                &"type value between 1 and 7 inclusive",
            )),
            None => Err(de::Error::missing_field("type")),
        }
    }
}
