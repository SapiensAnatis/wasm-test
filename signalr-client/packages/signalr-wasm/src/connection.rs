use crate::log::*;
use futures::Future;
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

pub struct Connection {
    url: String,
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

pub struct HandshakeFuture {
    websocket: WebSocket,
    on_message: Closure<dyn FnMut(MessageEvent) -> ()>,
    on_open: Closure<dyn FnMut() -> ()>,
    status: Rc<RefCell<HandshakeStatus>>,
    waker: Rc<OnceCell<Waker>>,
}

impl HandshakeFuture {
    pub fn new(websocket: WebSocket) -> Self {
        let status_rc = Rc::new(RefCell::new(HandshakeStatus::WaitingToConnect));
        let status_rc_clone = status_rc.clone();
        let status_rc_clone_2 = status_rc.clone();

        let waker_rc = Rc::new(OnceCell::new());
        let waker_rc_clone = waker_rc.clone();
        let waker_rc_clone_2 = waker_rc.clone();

        let on_message: Closure<dyn FnMut(_)> = Closure::new(move |e: MessageEvent| {
            Self::on_message(e, status_rc_clone.borrow_mut(), waker_rc_clone.get())
        });
        let on_connect: Closure<dyn FnMut()> = Closure::new(move || {
            Self::on_open(status_rc_clone_2.borrow_mut(), waker_rc_clone_2.get());
        });

        Self {
            websocket,
            status: status_rc,
            on_message,
            on_open: on_connect,
            waker: waker_rc,
        }
    }

    fn on_message(e: MessageEvent, mut status: RefMut<HandshakeStatus>, waker_opt: Option<&Waker>) {
        if let Ok(_abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            unimplemented!("Array buffers are not implemented");
        } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
            unimplemented!("Blobs are not implemented");
        } else if let Ok(string) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("message received: {}", string);
        } else {
            unimplemented!("What the hell");
        }

        *status = HandshakeStatus::Complete;

        assert!(waker_opt.is_some(), "waker not initialized");

        if let Some(waker) = waker_opt {
            console_log!("Waking up poll()...");
            waker.wake_by_ref();
        } else {
            console_error!("callback: Waker not initialized");
        }
    }

    fn on_open(mut status: RefMut<HandshakeStatus>, waker_opt: Option<&Waker>) {
        console_log!("Websocket connected!");

        *status = HandshakeStatus::NotStarted;

        if let Some(waker) = waker_opt {
            console_log!("Waking up poll()...");
            waker.wake_by_ref();
        } else {
            console_error!("callback: Waker not initialized");
        }
    }

    fn do_handshake(self: &Self) -> Result<(), String> {
        self.status.replace(HandshakeStatus::InProgress);

        console_log!("Sending handshake request");

        self.websocket
            .send_with_str("{ \"protocol\": \"json\", \"version\": 1  }\x1E")
            .map_err(|e| format!("Failed to send handshake: {:?}", e))?;

        Ok(())
    }
}

impl Future for HandshakeFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        _ = self.waker.get_or_init(|| cx.waker().clone());

        if self.websocket.onmessage().is_none() {
            self.websocket
                .set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
        }

        if self.websocket.onopen().is_none() {
            self.websocket
                .set_onopen(Some(self.on_open.as_ref().unchecked_ref()));
        }

        let status = *RefCell::borrow(&self.status);
        console_log!("Polled status: {}", status);

        match status {
            HandshakeStatus::WaitingToConnect => Poll::Pending,
            HandshakeStatus::NotStarted => {
                if let Err(e) = self.borrow().do_handshake() {
                    return Poll::Ready(Err(e));
                }

                Poll::Pending
            }
            HandshakeStatus::InProgress => Poll::Pending,
            HandshakeStatus::Complete => Poll::Ready(Ok(())),
            HandshakeStatus::Error => Poll::Ready(Err(String::from("Handshake failed"))),
        }
    }
}
