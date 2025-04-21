use std::cell::RefMut;
use futures::channel::oneshot::Sender;
use futures::StreamExt;
use wasm_bindgen_futures::spawn_local;
use crate::connection::{SignalRConnection, SubscriberMap};
use crate::message::{CompletionMessage, SignalRMessage};


impl SignalRConnection {
    pub fn start_reader(&mut self) -> Result<(), String> {
        console_log!("Starting read loop");

        let mut receiver = self.open_message_channel()?;
        let subscribers_clone = self.invocation_subscribers.clone();

        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                console_log!("Received message: {}", message);

                match serde_json::from_str(&message) {
                    Ok(SignalRMessage::Completion(m)) => {
                        if let Err(e) = Self::handle_completion(m, subscribers_clone.borrow_mut()) {
                            console_error!("{}", e);
                        }
                    }
                    Ok(SignalRMessage::Invocation(_)) => {
                        console_log!("Received invocation");
                    }
                    Ok(SignalRMessage::Ping) => {
                        console_log!("Pong!");
                    }
                    Err(e) => {
                        console_error!("Failed to deserialize message: {}", e);
                    }
                };
            }
        });

        Ok(())
    }

    pub(super) fn handle_completion(
        message: CompletionMessage,
        mut subscribers: RefMut<SubscriberMap>,
    ) -> Result<(), String> {
        

        // TODO: Removing here won't work for streaming invocations, it should be the invocation
        // caller who removes their own sender from the map.
        let sender: Sender<SignalRMessage> = match subscribers.remove(&message.invocation_id) {
            Some(s) => s,
            None => {
                return Err(format!(
                    "Failed to find subscriber for invocation ID {}",
                    message.invocation_id
                ))
            }
        };

        sender
            .send(SignalRMessage::Completion(message))
            .map_err(|_| "Failed to send subscriber message to subscriber".to_string())
    }
}