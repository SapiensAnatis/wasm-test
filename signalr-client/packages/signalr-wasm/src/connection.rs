use crate::log::*;
use futures::Future;
use futures_channel::mpsc::*;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cell::{Cell, OnceCell};
use std::convert::TryInto;
use std::fmt;
use std::num::Saturating;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Waker;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

enum WebSocketEvent {
    Open,
    Message(Vec<u8>),
}

pub struct Connection {
    url: String,
    event_receiver: Receiver<WebSocketEvent>,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }

    pub fn connect(self: &mut Self) -> Result<HandshakeFuture, &'static str> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err("Failed to create websocket");
            }
        };

        let fut = HandshakeFuture::new(ws);

        return Ok(fut);
    }
}

#[derive(Copy, Clone)]
enum HandshakeStatus {
    WaitingToConnect,
    NotStarted,
    InProgress,
    Complete,
    Error,
}

impl fmt::Display for HandshakeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandshakeStatus::WaitingToConnect => write!(f, "WAITING_TO_CONNECT"),
            HandshakeStatus::NotStarted => write!(f, "NOT_STARTED"),
            HandshakeStatus::InProgress => write!(f, "IN_PROGRESS"),
            HandshakeStatus::Complete => write!(f, "COMPLETE"),
            HandshakeStatus::Error => write!(f, "ERROR"),
        }
    }
}
