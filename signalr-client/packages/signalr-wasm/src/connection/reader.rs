use crate::connection::{CompletionSubscriberMap, InvocationSubscriberMap, SignalRConnection};
use crate::message::{CompletionMessage, InvocationMessage, SignalRMessage};
use futures::SinkExt;
use futures::StreamExt;
use std::cell::RefMut;
use wasm_bindgen_futures::spawn_local;

impl SignalRConnection {
    pub fn start_reader(&mut self) -> Result<(), String> {
        console_log!("Starting read loop");

        let mut receiver = self.open_message_channel()?;
        let cmp_subscribers_clone = self.completion_subscribers.clone();
        let inv_subscribers_clone = self.invocation_subscribers.clone();

        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                console_log!("Received message: {}", message);

                match serde_json::from_str(&message) {
                    Ok(SignalRMessage::Completion(m)) => {
                        if let Err(e) =
                            Self::handle_completion(m, cmp_subscribers_clone.borrow_mut()).await
                        {
                            console_error!("{}", e);
                        }
                    }
                    Ok(SignalRMessage::Invocation(m)) => {
                        if let Err(e) =
                            Self::handle_invocation(m, inv_subscribers_clone.borrow_mut()).await
                        {
                            console_error!("{}", e);
                        }
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

    pub(super) async fn handle_completion(
        message: CompletionMessage,
        mut subscribers: RefMut<'_, CompletionSubscriberMap>,
    ) -> Result<(), String> {
        let sender = match subscribers.get_mut(&message.invocation_id) {
            Some(s) => s,
            None => {
                return Err(format!(
                    "Failed to find subscriber for invocation ID {}",
                    message.invocation_id
                ))
            }
        };

        sender
            .send(message)
            .await
            .map_err(|_| "Failed to send subscriber message to subscriber".to_string())
    }

    pub(super) async fn handle_invocation(
        message: InvocationMessage,
        mut subscribers: RefMut<'_, InvocationSubscriberMap>,
    ) -> Result<(), String> {
        let sender = match subscribers.get_mut(&message.target) {
            Some(s) => s,
            None => {
                console_log!(
                    "No handler registered for invocation target {}",
                    message.target
                );
                return Ok(());
            }
        };

        sender
            .send(message)
            .await
            .map_err(|_| "Failed to send subscriber message to subscriber".to_string())
    }
}
