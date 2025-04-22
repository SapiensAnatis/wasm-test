use serde::{
    de::{self, Unexpected},
    Deserialize, Serialize,
};
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompletionMessage {
    r#type: u64,
    pub invocation_id: String,
    pub result: Value,
    pub error: Option<String>,
}

impl CompletionMessage {
    const TYPE: u64 = 3;

    pub fn new(invocation_id: String, result: Value, error: Option<String>) -> Self {
        Self {
            r#type: CompletionMessage::TYPE,
            invocation_id,
            result,
            error,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InvocationMessage {
    r#type: u64,
    #[serde(skip_deserializing)] // asp.net doesn't send them in invocations to us for some reason?
    pub invocation_id: String,
    pub target: String,
    pub arguments: Vec<Value>,
}

impl InvocationMessage {
    const TYPE: u64 = 1;
    pub fn new(invocation_id: String, target: String, arguments: Vec<Value>) -> Self {
        Self {
            r#type: InvocationMessage::TYPE,
            invocation_id,
            target,
            arguments,
        }
    }
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
    // TODO: this implementation could likely be more efficient, maybe check what serde generates
    // for tagged enums and copy it but replace the tag logic?
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;

        match value.get("type").and_then(Value::as_u64) {
            Some(InvocationMessage::TYPE) => {
                let inner_message = InvocationMessage::deserialize(value).map_err(|_| {
                    de::Error::invalid_type(Unexpected::StructVariant, &"an InvocationMessage")
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
