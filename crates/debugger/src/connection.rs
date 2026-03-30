use std::{future::Future, pin::Pin};

pub use tokio::io::AsyncRead;
use tokio::io::{AsyncWrite, AsyncWriteExt as _};

#[async_trait::async_trait]
pub trait PacketSender {
    async fn send(&mut self, data: &[u8]) -> anyhow::Result<()>;
}

pub type BoxedPacketReceiver = Pin<Box<dyn AsyncRead + Send>>;
pub type BoxedPacketSender = Box<dyn PacketSender + Send>;

#[async_trait::async_trait]
impl<T: AsyncWrite + Send + Unpin> PacketSender for T {
    async fn send(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.write_all(data).await?;
        Ok(())
    }
}

pub type ConnectionFactory = Box<
    dyn FnOnce() -> Pin<
        Box<dyn Future<Output = anyhow::Result<(BoxedPacketReceiver, BoxedPacketSender)>>>,
    >,
>;

pub(crate) struct Connection {
    sender: BoxedPacketSender,
    buffer: Vec<u8>,
    needs_flush: bool,
}

impl Connection {
    pub fn new(sender: BoxedPacketSender) -> Self {
        Self {
            sender,
            buffer: Vec::new(),
            needs_flush: false,
        }
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        if self.needs_flush {
            // eprintln!("");
            // eprintln!("-> to client");
            // eprintln!("{}", String::from_utf8_lossy(&self.buffer));
            // eprintln!("<- from client");

            tokio::runtime::Handle::current()
                .block_on(async { self.sender.send(&self.buffer).await })?;

            self.buffer.clear();
            self.needs_flush = false;
        }
        Ok(())
    }
}

impl gdbstub::conn::Connection for Connection {
    type Error = anyhow::Error;

    fn write_all(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        std::io::Write::write_all(&mut self.buffer, buf)?;
        Ok(())
    }

    fn on_session_start(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn write(&mut self, byte: u8) -> anyhow::Result<()> {
        self.buffer.push(byte);
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.needs_flush = true;
        Ok(())
    }
}

pub fn tokio_tcp_connection(port: u16) -> ConnectionFactory {
    Box::new(move || {
        Box::pin(async move {
            let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
            eprintln!("Awaiting for incoming GDB Remote connection on port {port}");
            let (tcp_stream, _addr) = tcp_listener.accept().await?;
            eprintln!("GDB Remote connection accepted!");
            let (read, write) = tcp_stream.into_split();
            let rx: BoxedPacketReceiver = Box::pin(read);
            let tx: BoxedPacketSender = Box::new(write);
            anyhow::Ok((rx, tx))
        })
    })
}
