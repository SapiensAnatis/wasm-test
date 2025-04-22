use crate::connection::{SignalRConnection, CHANNEL_BOUND_SIZE};
use crate::message::{CompletionMessage, InvocationMessage};
use futures::channel::mpsc;
use futures::StreamExt;
use serde_json::Value;
use web_sys::WebSocket;

impl SignalRConnection {
    pub async fn send_invocation(
        &mut self,
        target: String,
        args: Vec<Value>,
    ) -> Result<(), String> {
        let ws: &WebSocket = match self.web_socket.get_mut() {
            Some(ws) => ws,
            None => {
                return Err("No open socket".to_owned());
            }
        };

        self.invocation_id += 1;

        let invocation = InvocationMessage::new(self.invocation_id.to_string(), target, args);

        Self::send_struct(ws, &invocation)
            .map_err(|e| format!("Failed to send message: {:?}", e))?;

        self.await_invocation_response(invocation.invocation_id)
            .await?;

        Ok(())
    }

    async fn await_invocation_response(&mut self, invocation_id: String) -> Result<(), String> {
        let (sender, mut receiver) = mpsc::channel::<CompletionMessage>(CHANNEL_BOUND_SIZE);

        {
            self.completion_subscribers
                .borrow_mut()
                .insert(invocation_id.clone(), sender);
        }

        console_log!("Waiting for response");

        // TODO: consider timeout
        let message = receiver.next().await;

        {
            self.completion_subscribers
                .borrow_mut()
                .remove(&invocation_id);
        }

        if message.is_none() {
            return Err("Failed to receive message".to_owned());
        }

        console_log!("Received invocation reply: {:?}", message);

        Ok(())
    }
}
