use std::collections::HashMap;
use std::fmt::Display;
use futures_util::{FutureExt, Sink, SinkExt};
use futures_util::stream::StreamExt;
use futures_util::select;
use resoxide_json::Json;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    protocol::WebSocketConfig,
    Message as WsMessage,
};
use crate::messages::Message;
use crate::responses::Response;

pub struct Client {
    tx: tokio::sync::mpsc::Sender<Command>,
    handle: Option<std::thread::JoinHandle<Result<()>>>,
    close_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    WebSocket(tokio_tungstenite::tungstenite::error::Error),
    Closed,
    Unknown,
    Json(resoxide_json::Error),
    BinaryMismatch,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Closed
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::Closed
    }
}

impl From<tokio_tungstenite::tungstenite::error::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::error::Error) -> Self {
        Self::WebSocket(e)
    }
}

impl From<resoxide_json::Error> for Error {
    fn from(e: resoxide_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Client {
    fn connect_impl(request: tokio_tungstenite::tungstenite::handshake::client::Request, close_rx: tokio::sync::oneshot::Receiver<()>, rx: tokio::sync::mpsc::Receiver<Command>) -> Result<(std::thread::JoinHandle<Result<()>>,tokio::sync::oneshot::Receiver<()>)> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        let handle = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async move {
                let conf = WebSocketConfig::default();
                let (websocket, _) = tokio_tungstenite::connect_async_with_config(request, Some(conf), true).await?;
                let (mut sink, stream) = websocket.split();
                let mut stream = stream.fuse();
                let mut rx = tokio_stream::wrappers::ReceiverStream::new(rx).fuse();
                let mut closer = close_rx.fuse();
                let mut counter = 0usize;
                let mut responders: HashMap<String, tokio::sync::oneshot::Sender<Response>> = HashMap::new();
                let _ = resp_tx.send(());
                loop {
                    select! {
                        msg = stream.next() => {
                            match msg {
                                None => return Err(Error::Closed),
                                Some(Err(e)) => return Err(Error::WebSocket(e)),
                                Some(Ok(WsMessage::Binary(_))) => {}
                                Some(Ok(WsMessage::Text(text))) => {
                                    let resp = Response::deserialize(text.as_str())?;
                                    if let Some(responder) = responders.remove(resp.message_id()) {
                                        let _ = responder.send(resp);
                                    }
                                },
                                Some(Ok(WsMessage::Close(_))) => return Ok(()),
                                _ => (),
                            }
                        },
                        cmd = rx.next() => {
                            match cmd {
                                None => return Err(Error::Closed),
                                Some(Command { msg, resp, data }) => {
                                    counter += 1;
                                    let msg_id = format!("Msg{}", counter);
                                    let msg = msg.with_message_id(msg_id.clone()).to_token()?.serialize()?;
                                    let ws_msg = WsMessage::text(msg);
                                    responders.insert(msg_id, resp);
                                    sink.send(ws_msg).await?;
                                    if let Some(binary) = data {
                                        sink.send(WsMessage::binary(binary)).await?;
                                    }
                                    sink.flush().await?;
                                }
                            }
                        },
                        closed = &mut closer => {
                            let mut websocket = sink.reunite(stream.into_inner()).unwrap();
                            websocket.close(None).await?;
                            return Ok(closed?);
                        }
                    }
                }
            })
        });
        Ok((handle, resp_rx))
    }

    pub async fn connect_port(port: u16) -> Result<Client> {
        let request = format!("ws://localhost:{}", port).into_client_request()?;
        let (tx, rx) = tokio::sync::mpsc::channel::<Command>(8);
        let (close_tx, close_rx) = tokio::sync::oneshot::channel();
        let (handle, resp_rx) = Self::connect_impl(request, close_rx, rx)?;
        match resp_rx.await {
            Ok(_) => Ok(Client { tx, handle: Some(handle), close_tx: Some(close_tx) }),
            Err(_) => {
                handle.join().unwrap()?;
                Err(Error::Unknown)
            }
        }
    }

    pub fn blocking_connect_port(port: u16) -> Result<Client> {
        let request = format!("ws://localhost:{}", port).into_client_request()?;
        let (tx, rx) = tokio::sync::mpsc::channel::<Command>(8);
        let (close_tx, close_rx) = tokio::sync::oneshot::channel();
        let (handle, resp_rx) = Self::connect_impl(request, close_rx, rx)?;
        match resp_rx.blocking_recv() {
            Ok(_) => Ok(Client { tx, handle: Some(handle), close_tx: Some(close_tx) }),
            Err(_) => {
                handle.join().unwrap()?;
                Err(Error::Unknown)
            }
        }
    }

    pub async fn close(mut self) -> Result<()> {
        if let Some(close_tx) = self.close_tx.take() {
            let _ = close_tx.send(());
        }
        self.tx.closed().await;
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap()?;
        }
        Ok(())
    }

    pub fn blocking_close(mut self) -> Result<()> {
        if let Some(close_tx) = self.close_tx.take() {
            let _ = close_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap()?;
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        if let Some(handle) = &self.handle {
            !handle.is_finished()
        } else {
            false
        }
    }

    pub async fn call(&self, msg: Message, data: Option<Vec<u8>>) -> Result<Response> {
        if msg.has_binary() != data.is_some() {
            return Err(Error::BinaryMismatch);
        }
        if self.tx.is_closed() {
            return Err(Error::Closed);
        }
        let (resp, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command { msg, resp, data }).await?;
        Ok(rx.await?)
    }

    pub fn blocking_call(&self, msg: Message, data: Option<Vec<u8>>) -> Result<Response> {
        if msg.has_binary() != data.is_some() {
            return Err(Error::BinaryMismatch);
        }
        if self.tx.is_closed() {
            return Err(Error::Closed);
        }
        let (resp, rx) = tokio::sync::oneshot::channel();
        self.tx.blocking_send(Command { msg, resp, data })?;
        Ok(rx.blocking_recv()?)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(close_tx) = self.close_tx.take() {
            let _ = close_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap().expect("Error dropping client without calling close()");
        }
    }
}

struct Command {
    msg: Message,
    resp: tokio::sync::oneshot::Sender<Response>,
    data: Option<Vec<u8>>,
}