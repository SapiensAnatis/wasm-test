use std::fmt;

use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor},
    Deserialize,
};
use serde_json::Value;

trait MessageType {
    const TYPE: u64;
}

#[derive(Deserialize)]
struct CompletionMessage {
    invocation_id: String,
    result: Value,
    error: Option<String>,
}

impl MessageType for CompletionMessage {
    const TYPE: u64 = 3;
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

enum SignalRMessage {
    Ping,
    Completion(CompletionMessage),
}

// Messages are  _almost_ an internally tagged enum, except Serde
// currently only supports the tag being the literal name of the enum,
// not a number like SignalR sends.
impl<'de> Deserialize<'de> for SignalRMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;

        match value.get("type").and_then(Value::as_u64) {
            Some(1) => unimplemented!("can't deserialize Invocation"),
            Some(2) => unimplemented!("can't deserialize StreamItem"),
            Some(CompletionMessage::TYPE) => {
                let inner_message = CompletionMessage::deserialize(value).map_err(|_| {
                    de::Error::invalid_value(Unexpected::StructVariant, &"a CompletionMessage")
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
