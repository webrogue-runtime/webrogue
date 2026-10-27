use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use bytes::Bytes;
use tokio::{
    io::AsyncReadExt as _,
    sync::{Notify, mpsc},
};
use wasmtime::component::Resource;
use wasmtime_wasi::p2::{
    DynInputStream, DynOutputStream, InputStream, OutputStream, Pollable, SocketResult,
    StreamError, StreamResult,
};

use crate::DebuggerHost;
use crate::connection::{BoxedPacketReceiver, BoxedPacketSender};

wasmtime::component::bindgen!({
    path: "wit",
    world: "sockets",
    imports: {
        "wasi:sockets/tcp.[method]tcp-socket.start-bind": async | trappable,
        "wasi:sockets/tcp.[method]tcp-socket.start-listen": async | trappable,
        "wasi:sockets/tcp.[method]tcp-socket.accept": async | trappable,
        "wasi:sockets/tcp-create-socket.create-tcp-socket": async | trappable,
        default: trappable,
    },
    trappable_error_type: {
        "wasi:io/streams.stream-error" => wasmtime_wasi::p2::StreamError,
        "wasi:sockets/network.error-code" => wasmtime_wasi::p2::SocketError,
    },
    with: {
        "wasi:io/poll": wasmtime_wasi::p2::bindings::io::poll,
        "wasi:io/streams": wasmtime_wasi::p2::bindings::io::streams,
        "wasi:io/error": wasmtime_wasi::p2::bindings::io::error,
        "wasi:sockets/network": wasmtime_wasi::p2::bindings::sockets::network,
        "wasi:sockets/tcp.tcp-socket": crate::wasi_sockets::VirtualTcpSocket,
    },
});

use self::wasi::sockets::tcp::{Host as TcpHost, HostTcpSocket, ShutdownType};
use self::wasi::sockets::tcp_create_socket::Host as TcpCreateSocketHost;
use wasmtime_wasi::p2::bindings::sockets::network::{
    ErrorCode, IpAddressFamily, IpSocketAddress, Network,
};

#[derive(Default)]
pub(crate) struct OutputState {
    pending: AtomicUsize,
    notify: Notify,
}

impl OutputState {
    fn add(&self) {
        self.pending.fetch_add(1, Ordering::SeqCst);
    }

    fn done(&self) {
        if self.pending.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.notify.notify_one();
        }
    }

    pub(crate) async fn wait_drained(&self) {
        loop {
            let notified = self.notify.notified();
            if self.pending.load(Ordering::SeqCst) == 0 {
                return;
            }
            notified.await;
        }
    }
}

pub struct VirtualTcpSocket {
    family: IpAddressFamily,
    local_address: Option<IpSocketAddress>,
    state: TcpState,
    pending: Option<(BoxedPacketReceiver, BoxedPacketSender)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TcpState {
    Unbound,
    Bound,
    Listening,
    Connected,
    Closed,
}

impl VirtualTcpSocket {
    fn new(family: IpAddressFamily) -> Self {
        Self {
            family,
            local_address: None,
            state: TcpState::Unbound,
            pending: None,
        }
    }

    fn connected(family: IpAddressFamily) -> Self {
        Self {
            family,
            local_address: None,
            state: TcpState::Connected,
            pending: None,
        }
    }
}

#[async_trait::async_trait]
impl Pollable for VirtualTcpSocket {
    async fn ready(&mut self) {}
}

impl TcpHost for DebuggerHost {}

impl self::wasi::sockets::network::Host for DebuggerHost {
    fn convert_error_code(
        &mut self,
        error: wasmtime_wasi::p2::SocketError,
    ) -> wasmtime::Result<ErrorCode> {
        error.downcast()
    }

    fn network_error_code(
        &mut self,
        _err: Resource<wasmtime_wasi::p2::IoError>,
    ) -> wasmtime::Result<Option<ErrorCode>> {
        Ok(None)
    }
}

impl self::wasi::sockets::network::HostNetwork for DebuggerHost {
    fn drop(&mut self, this: Resource<Network>) -> wasmtime::Result<()> {
        let _ = self.table.delete(this)?;
        Ok(())
    }
}

impl TcpCreateSocketHost for DebuggerHost {
    async fn create_tcp_socket(
        &mut self,
        family: IpAddressFamily,
    ) -> SocketResult<Resource<VirtualTcpSocket>> {
        self.table
            .push(VirtualTcpSocket::new(family))
            .map_err(|_| ErrorCode::NewSocketLimit.into())
    }
}

impl HostTcpSocket for DebuggerHost {
    async fn start_bind(
        &mut self,
        this: Resource<VirtualTcpSocket>,
        _network: Resource<Network>,
        local_address: IpSocketAddress,
    ) -> SocketResult<()> {
        let socket = self
            .table
            .get_mut(&this)
            .map_err(|_| ErrorCode::InvalidState)?;
        socket.local_address = Some(local_address);
        socket.state = TcpState::Bound;
        Ok(())
    }

    fn finish_bind(&mut self, this: Resource<VirtualTcpSocket>) -> SocketResult<()> {
        let socket = self
            .table
            .get_mut(&this)
            .map_err(|_| ErrorCode::InvalidState)?;
        if socket.state != TcpState::Bound {
            return Err(ErrorCode::NotInProgress.into());
        }
        Ok(())
    }

    fn start_connect(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _network: Resource<Network>,
        _remote_address: IpSocketAddress,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn finish_connect(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
    ) -> SocketResult<(Resource<DynInputStream>, Resource<DynOutputStream>)> {
        Err(ErrorCode::NotSupported.into())
    }

    async fn start_listen(&mut self, this: Resource<VirtualTcpSocket>) -> SocketResult<()> {
        let Some(connection_factory) = self.connection_factory.take() else {
            return Err(ErrorCode::NotSupported.into());
        };

        let (receiver, sender) = connection_factory
            .connect()
            .await
            .map_err(|_| ErrorCode::Unknown)?;

        let socket = self
            .table
            .get_mut(&this)
            .map_err(|_| ErrorCode::InvalidState)?;
        socket.pending = Some((receiver, sender));
        socket.state = TcpState::Listening;
        Ok(())
    }

    fn finish_listen(&mut self, this: Resource<VirtualTcpSocket>) -> SocketResult<()> {
        let socket = self
            .table
            .get_mut(&this)
            .map_err(|_| ErrorCode::InvalidState)?;
        if socket.state != TcpState::Listening {
            return Err(ErrorCode::NotInProgress.into());
        }
        Ok(())
    }

    async fn accept(
        &mut self,
        this: Resource<VirtualTcpSocket>,
    ) -> SocketResult<(
        Resource<VirtualTcpSocket>,
        Resource<DynInputStream>,
        Resource<DynOutputStream>,
    )> {
        let (family, (receiver, sender)) = {
            let socket = self
                .table
                .get_mut(&this)
                .map_err(|_| ErrorCode::InvalidState)?;
            if socket.state != TcpState::Listening {
                return Err(ErrorCode::InvalidState.into());
            }
            let connection = socket.pending.take().ok_or(ErrorCode::WouldBlock)?;
            (socket.family, connection)
        };

        let accepted = self
            .table
            .push(VirtualTcpSocket::connected(family))
            .map_err(|_| ErrorCode::NewSocketLimit)?;

        let input: DynInputStream = Box::new(VirtualInput::new(receiver));
        let output: DynOutputStream =
            Box::new(VirtualOutput::new(sender, self.output_state.clone()));
        let input = self
            .table
            .push_child(input, &accepted)
            .map_err(|_| ErrorCode::Unknown)?;
        let output = self
            .table
            .push_child(output, &accepted)
            .map_err(|_| ErrorCode::Unknown)?;

        Ok((accepted, input, output))
    }

    fn local_address(&mut self, this: Resource<VirtualTcpSocket>) -> SocketResult<IpSocketAddress> {
        let socket = self.table.get(&this).map_err(|_| ErrorCode::InvalidState)?;
        socket
            .local_address
            .ok_or_else(|| ErrorCode::InvalidState.into())
    }

    fn remote_address(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
    ) -> SocketResult<IpSocketAddress> {
        Err(ErrorCode::InvalidState.into())
    }

    fn is_listening(&mut self, this: Resource<VirtualTcpSocket>) -> wasmtime::Result<bool> {
        let socket = self.table.get(&this)?;
        Ok(socket.state == TcpState::Listening)
    }

    fn address_family(
        &mut self,
        this: Resource<VirtualTcpSocket>,
    ) -> wasmtime::Result<IpAddressFamily> {
        let socket = self.table.get(&this)?;
        Ok(socket.family)
    }

    fn set_listen_backlog_size(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn keep_alive_enabled(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<bool> {
        Ok(false)
    }

    fn set_keep_alive_enabled(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: bool,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn keep_alive_idle_time(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u64> {
        Ok(0)
    }

    fn set_keep_alive_idle_time(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn keep_alive_interval(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u64> {
        Ok(0)
    }

    fn set_keep_alive_interval(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn keep_alive_count(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u32> {
        Ok(0)
    }

    fn set_keep_alive_count(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u32,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn hop_limit(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u8> {
        Ok(64)
    }

    fn set_hop_limit(&mut self, _this: Resource<VirtualTcpSocket>, _value: u8) -> SocketResult<()> {
        Ok(())
    }

    fn receive_buffer_size(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u64> {
        Ok(64 * 1024)
    }

    fn set_receive_buffer_size(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn send_buffer_size(&mut self, _this: Resource<VirtualTcpSocket>) -> SocketResult<u64> {
        Ok(64 * 1024)
    }

    fn set_send_buffer_size(
        &mut self,
        _this: Resource<VirtualTcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Ok(())
    }

    fn subscribe(
        &mut self,
        this: Resource<VirtualTcpSocket>,
    ) -> wasmtime::Result<Resource<wasmtime_wasi::p2::DynPollable>> {
        wasmtime_wasi::p2::subscribe(&mut self.table, this)
    }

    fn shutdown(
        &mut self,
        this: Resource<VirtualTcpSocket>,
        _shutdown_type: ShutdownType,
    ) -> SocketResult<()> {
        let socket = self
            .table
            .get_mut(&this)
            .map_err(|_| ErrorCode::InvalidState)?;
        if socket.state != TcpState::Connected {
            return Err(ErrorCode::InvalidState.into());
        }
        socket.state = TcpState::Closed;
        Ok(())
    }

    fn drop(&mut self, this: Resource<VirtualTcpSocket>) -> wasmtime::Result<()> {
        let _ = self.table.delete(this)?;
        Ok(())
    }
}

/// An `InputStream` backed by a `ConnectionFactory` receiver.
struct VirtualInput {
    receiver: BoxedPacketReceiver,
    buffer: Vec<u8>,
    offset: usize,
    eof: bool,
    injected_interrupt: bool,
}

impl VirtualInput {
    fn new(receiver: BoxedPacketReceiver) -> Self {
        Self {
            receiver,
            buffer: Vec::new(),
            offset: 0,
            eof: false,
            injected_interrupt: false,
        }
    }

    async fn on_eof(&mut self) {
        if !self.injected_interrupt {
            self.buffer.clear();
            self.buffer.push(0x03);
            self.offset = 0;
            self.injected_interrupt = true;
            return;
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        self.eof = true;
    }
}

#[async_trait::async_trait]
impl Pollable for VirtualInput {
    async fn ready(&mut self) {
        if self.offset < self.buffer.len() || self.eof {
            return;
        }

        let mut buf = [0u8; 8192];
        match self.receiver.read(&mut buf).await {
            Ok(0) => self.on_eof().await,
            Ok(n) => {
                self.buffer.clear();
                self.buffer.extend_from_slice(&buf[..n]);
                self.offset = 0;
            }
            Err(_) => self.on_eof().await,
        }
    }
}

impl InputStream for VirtualInput {
    fn read(&mut self, size: usize) -> StreamResult<Bytes> {
        if self.offset >= self.buffer.len() {
            return if self.eof {
                Err(StreamError::Closed)
            } else {
                Ok(Bytes::new())
            };
        }

        let end = (self.offset + size).min(self.buffer.len());
        let bytes = Bytes::copy_from_slice(&self.buffer[self.offset..end]);
        self.offset = end;
        if self.offset == self.buffer.len() {
            self.buffer.clear();
            self.offset = 0;
        }
        Ok(bytes)
    }
}

struct VirtualOutput {
    sender: mpsc::UnboundedSender<Vec<u8>>,
    closed: Arc<AtomicBool>,
    state: Arc<OutputState>,
}

impl VirtualOutput {
    fn new(mut sender: BoxedPacketSender, state: Arc<OutputState>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let closed = Arc::new(AtomicBool::new(false));
        let task_closed = closed.clone();
        let task_state = state.clone();

        tokio::spawn(async move {
            let mut failed = false;
            while let Some(chunk) = rx.recv().await {
                if failed {
                    // Keep the pending counter balanced so `wait_drained`
                    // does not have to wait for its timeout.
                    task_state.done();
                    continue;
                }
                let sent = sender.send(&chunk).await.is_ok();
                task_state.done();
                if !sent {
                    task_closed.store(true, Ordering::SeqCst);
                    failed = true;
                }
            }
        });

        Self {
            sender: tx,
            closed,
            state,
        }
    }
}

#[async_trait::async_trait]
impl Pollable for VirtualOutput {
    async fn ready(&mut self) {}
}

impl OutputStream for VirtualOutput {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(StreamError::Closed);
        }
        self.state.add();
        if self.sender.send(bytes.to_vec()).is_err() {
            self.state.done();
            return Err(StreamError::Closed);
        }
        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        if self.closed.load(Ordering::SeqCst) {
            Err(StreamError::Closed)
        } else {
            Ok(64 * 1024)
        }
    }
}

pub(crate) fn add_to_linker(
    linker: &mut wasmtime::component::Linker<DebuggerHost>,
) -> wasmtime::Result<()> {
    linker.allow_shadowing(true);
    self::wasi::sockets::tcp::add_to_linker::<_, wasmtime::component::HasSelf<DebuggerHost>>(
        linker,
        |host| host,
    )?;
    self::wasi::sockets::tcp_create_socket::add_to_linker::<
        _,
        wasmtime::component::HasSelf<DebuggerHost>,
    >(linker, |host| host)?;
    linker.allow_shadowing(false);
    Ok(())
}
