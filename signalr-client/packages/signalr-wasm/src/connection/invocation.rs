use crate::connection::{SignalRConnection, SubscriberMap};
use crate::message::{CompletionMessage, InvocationMessage, SignalRMessage};
use futures::channel::oneshot;
use web_sys::WebSocket;

impl SignalRConnection {
    pub async fn send_invocation(&mut self, target: String, args: Vec<String>) -> Result<(), String> {
        let ws: &WebSocket = match self.web_socket.get_mut() {
            Some(ws) => ws,
            None => {
                return Err("No open socket".to_owned());
            }
        };

        self.invocation_id += 1;
        let invocation_id_str = self.invocation_id.to_string();

        let invocation = InvocationMessage::new(invocation_id_str, target, args);

        Self::send_struct(&ws, &invocation)
            .map_err(|e| format!("Failed to send message: {:?}", e))?;

        self.await_response(invocation.invocation_id).await?;

        Ok(())
    }

    async fn await_response(&mut self, invocation_id: String) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel::<SignalRMessage>();

        {
            self.invocation_subscribers
                .borrow_mut()
                .insert(invocation_id, sender);
        }

        // TODO: consider timeout
        let message = receiver
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        console_log!("Received invocation reply: {:?}", message);

        Ok(())
    }
}
