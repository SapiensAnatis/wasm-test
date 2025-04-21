use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use web_sys::{MessageEvent, WebSocket};
use crate::connection::SignalRConnection;
use wasm_bindgen::JsCast;

#[derive(Serialize)]
struct HandshakeRequest {
    protocol: &'static str,
    version: u8,
}

#[derive(Deserialize)]
struct HandshakeResponse {
    error: Option<String>,
}

impl SignalRConnection {
    pub async fn connect(&mut self) -> Result<(), String> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err(String::from("Failed to create websocket"));
            }
        };

        let (open_sender, open_receiver) = oneshot::channel::<()>();
        let (handshake_sender, handshake_receiver) = oneshot::channel::<Result<(), String>>();

        let on_open = Closure::once(move || {
            if let Err(e) = open_sender.send(()) {
                console_error!("Failed to send open message: {:?}", e);
            }
        });

        let on_message = Closure::once(move |e: MessageEvent| {
            let parse_result = Self::parse_message(&e);

            /* TODO: improve error handling here - can we avoid expect / panics? */

            let message: String = match parse_result {
                Ok(mut msg) => msg.remove(0),
                Err(e) => {
                    handshake_sender
                        .send(Err(e))
                        .expect("Failed to send on_message error");
                    return;
                }
            };

            console_log!("Received handshake response: {}", message);

            let parsed_result: HandshakeResponse = match serde_json::from_str(&message) {
                Ok(val) => val,
                Err(e) => {
                    handshake_sender
                        .send(Err(format!("Failed to parse JSON: {}", e)))
                        .expect("Failed to send on_message error");
                    return;
                }
            };

            if let Some(error) = parsed_result.error {
                handshake_sender
                    .send(Err(format!("Received handshake error: {}", error)))
                    .expect("Failed to send on_message error");
                return;
            }

            handshake_sender
                .send(Ok(()))
                .expect("Failed to send on_message success");
        });

        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        match open_receiver.await {
            Ok(()) => {
                console_log!("Received open event, transmitting handshake...");
            }
            Err(e) => return Err(format!("Failed to get open event: {}", e)),
        }

        let request = HandshakeRequest { protocol: "json", version: 1};
        if let Err(e) = Self::send_struct(&ws, &request) {
            return Err(format!("Failed to send handshake: {:?}", e));
        }

        match handshake_receiver.await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("Handshake failed: {}", e));
                }

                console_log!("Successfully established connection");
            }
            Err(e) => return Err(format!("Failed to get handshake event: {}", e)),
        }

        ws.set_onopen(None);
        ws.set_onmessage(None);


        // TODO: handle gracefully
        self.web_socket.set(ws).unwrap();

        self.start_reader()?;


        Ok(())
    }
}
