use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Context as _;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use wasmtime::{Config, Engine, Instance, Module, Store};
use webrogue_debugger::connection::{premade_connection, tokio_tcp_connection};

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_stops_paused_guest_on_disconnect() -> anyhow::Result<()> {
    let (engine, module) = test_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();

    let port = free_port()?;

    let debugger = webrogue_debugger::debug(store, tokio_tcp_connection(port), move |store| {
        Box::pin(async move {
            main.call_async(store, &[], &mut [])
                .await
                .map_err(anyhow::Error::from)?;
            ran_clone.store(true, Ordering::SeqCst);
            Ok(())
        })
    });

    let connect_and_disconnect = async {
        let stream = connect_with_retry(port).await?;
        drop(stream);
        anyhow::Ok(())
    };

    let (debugger_result, connect_result) = tokio::join!(debugger, connect_and_disconnect);
    connect_result?;
    debugger_result?;
    assert!(
        !ran.load(Ordering::SeqCst),
        "guest ran after the debugger disconnected from its initial stop"
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_speaks_gdb_remote() -> anyhow::Result<()> {
    let (engine, module) = test_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();
    let port = free_port()?;

    let debugger = webrogue_debugger::debug(store, tokio_tcp_connection(port), move |store| {
        Box::pin(async move {
            main.call_async(store, &[], &mut [])
                .await
                .map_err(anyhow::Error::from)?;
            ran_clone.store(true, Ordering::SeqCst);
            Ok(())
        })
    });

    let gdb_session = async {
        let mut stream = connect_with_retry(port).await?;
        run_gdb_session(&mut stream).await
    };

    let (debugger_result, gdb_result) = tokio::join!(debugger, gdb_session);
    gdb_result?;
    debugger_result?;
    assert!(ran.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_forwards_interrupts() -> anyhow::Result<()> {
    let (engine, module) = long_running_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let (client, server) = tokio::io::duplex(4096);
    let (server_rx, server_tx) = tokio::io::split(server);

    let debugger = webrogue_debugger::debug(
        store,
        premade_connection(Box::new(server_tx), Box::pin(server_rx), || {}),
        move |store| {
            Box::pin(async move {
                main.call_async(store, &[], &mut [])
                    .await
                    .map_err(anyhow::Error::from)
            })
        },
    );

    let gdb_session = async {
        let mut client = client;

        send_packet(&mut client, "?").await.context("send ?")?;
        let stop = read_packet(&mut client).await.context("read ?")?;
        assert!(stop.starts_with('T') || stop.starts_with('S'));

        // Start the long-running debuggee, then interrupt it with a raw
        // Ctrl-C byte (not a GDB packet).
        send_packet(&mut client, "c").await.context("send c")?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        client.write_all(&[0x03]).await.context("send Ctrl-C")?;
        client.flush().await?;

        let stop = read_packet(&mut client).await.context("read interrupt")?;
        assert!(
            stop.starts_with("S02") || stop.starts_with("T02"),
            "unexpected interrupt stop reply: {stop}"
        );

        // The debuggee must still be resumable after the stop.
        send_packet(&mut client, "c")
            .await
            .context("send c again")?;
        let stop = read_packet(&mut client).await.context("read exit")?;
        assert!(
            stop.starts_with('W') || stop.starts_with('X'),
            "unexpected exit reply: {stop}"
        );
        anyhow::Ok(())
    };

    let (debugger_result, gdb_result) = tokio::join!(debugger, gdb_session);
    gdb_result?;
    debugger_result?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_survives_disconnect_while_running() -> anyhow::Result<()> {
    let (engine, module) = long_running_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let (client, server) = tokio::io::duplex(4096);
    let (server_rx, server_tx) = tokio::io::split(server);

    let debugger = webrogue_debugger::debug(
        store,
        premade_connection(Box::new(server_tx), Box::pin(server_rx), || {}),
        move |store| {
            Box::pin(async move {
                main.call_async(store, &[], &mut [])
                    .await
                    .map_err(anyhow::Error::from)
            })
        },
    );

    let gdb_session = async {
        let mut client = client;
        send_packet(&mut client, "?").await.context("send ?")?;
        let stop = read_packet(&mut client).await.context("read ?")?;
        assert!(stop.starts_with('T') || stop.starts_with('S'));

        send_packet(&mut client, "c").await.context("send c")?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        // Disconnect without interrupting or detaching first.
        drop(client);
        anyhow::Ok(())
    };

    let (debugger_result, gdb_result) = tokio::join!(debugger, gdb_session);
    gdb_result?;
    debugger_result?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_stops_guest_on_disconnect() -> anyhow::Result<()> {
    let (engine, module) = non_terminating_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();

    let (client, server) = tokio::io::duplex(4096);
    let (server_rx, server_tx) = tokio::io::split(server);

    let debugger = webrogue_debugger::debug(
        store,
        premade_connection(Box::new(server_tx), Box::pin(server_rx), || {}),
        move |store| {
            Box::pin(async move {
                main.call_async(store, &[], &mut [])
                    .await
                    .map_err(anyhow::Error::from)?;
                ran_clone.store(true, Ordering::SeqCst);
                Ok(())
            })
        },
    );

    let gdb_session = async {
        let mut client = client;
        send_packet(&mut client, "?").await.context("send ?")?;
        let stop = read_packet(&mut client).await.context("read ?")?;
        assert!(stop.starts_with('T') || stop.starts_with('S'));

        send_packet(&mut client, "c").await.context("send c")?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        // Disconnect while the guest is executing Wasm.
        drop(client);
        anyhow::Ok(())
    };

    let (debugger_result, gdb_result) =
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            tokio::join!(debugger, gdb_session)
        })
        .await
        .context("debugger did not stop the guest after disconnect")?;

    gdb_result?;
    debugger_result?;
    assert!(
        !ran.load(Ordering::SeqCst),
        "guest ran to completion after the debugger disconnected"
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn debugger_component_proxies_premade_connection() -> anyhow::Result<()> {
    let (engine, module) = test_module().await?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new_async(&mut store, &module, &[]).await?;
    let main = instance.get_func(&mut store, "main").unwrap();

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();
    let constructed = Arc::new(AtomicBool::new(false));
    let constructed_clone = constructed.clone();

    let (client, server) = tokio::io::duplex(4096);
    let (server_rx, server_tx) = tokio::io::split(server);

    let debugger = webrogue_debugger::debug(
        store,
        premade_connection(Box::new(server_tx), Box::pin(server_rx), move || {
            constructed_clone.store(true, Ordering::SeqCst)
        }),
        move |store| {
            Box::pin(async move {
                main.call_async(store, &[], &mut [])
                    .await
                    .map_err(anyhow::Error::from)?;
                ran_clone.store(true, Ordering::SeqCst);
                Ok(())
            })
        },
    );

    let gdb_session = async {
        let mut client = client;
        run_gdb_session(&mut client).await
    };

    let (debugger_result, gdb_result) = tokio::join!(debugger, gdb_session);
    gdb_result?;
    debugger_result?;
    assert!(constructed.load(Ordering::SeqCst));
    assert!(ran.load(Ordering::SeqCst));
    Ok(())
}

async fn run_gdb_session<S>(stream: &mut S) -> anyhow::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // The debugger starts at the initial stop.
    send_packet(stream, "?").await.context("send ?")?;
    let stop = read_packet(stream).await.context("read ?")?;
    assert!(
        stop.starts_with('T') || stop.starts_with('S'),
        "unexpected stop reply: {stop}"
    );

    // The initial stop must already report the instantiated Wasm modules.
    // LLDB fetches the library list only once, at attach time; if it is empty
    // here, the client never loads the modules or their DWARF, and backtraces
    // are rendered as raw addresses.
    send_packet(stream, "qXfer:libraries:read::0,fff")
        .await
        .context("send libraries")?;
    let libraries = read_packet(stream).await.context("read libraries")?;
    assert!(
        libraries.contains("<section address="),
        "initial library list was empty: {libraries}"
    );

    // Run the debuggee to completion. The debugger disconnects after
    // reporting the exit.
    send_packet(stream, "c").await.context("send c")?;
    let stop = read_packet(stream).await.context("read c")?;
    assert!(
        stop.starts_with('W') || stop.starts_with('X'),
        "unexpected exit reply: {stop}"
    );
    Ok(())
}

async fn test_module() -> anyhow::Result<(Engine, Module)> {
    test_module_with_iterations(1000).await
}

async fn long_running_module() -> anyhow::Result<(Engine, Module)> {
    test_module_with_iterations(2_000_000_000).await
}

async fn non_terminating_module() -> anyhow::Result<(Engine, Module)> {
    let mut config = Config::new();
    config.guest_debug(true).epoch_interruption(true);
    let engine = Engine::new(&config)?;
    let module = Module::new(
        &engine,
        r#"
            (module
              (func (export "main")
                (loop
                  (br 0))))
        "#,
    )?;
    Ok((engine, module))
}

async fn test_module_with_iterations(iterations: u32) -> anyhow::Result<(Engine, Module)> {
    let mut config = Config::new();
    config.guest_debug(true).epoch_interruption(true);
    let engine = Engine::new(&config)?;
    let module = Module::new(
        &engine,
        format!(
            r#"
            (module
              (func (export "main")
                (local i32)
                (local.set 0 (i32.const {iterations}))
                (loop
                  (local.set 0 (i32.sub (local.get 0) (i32.const 1)))
                  (br_if 0 (local.get 0)))))
        "#
        ),
    )?;
    Ok((engine, module))
}

fn free_port() -> anyhow::Result<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

async fn connect_with_retry(port: u16) -> anyhow::Result<tokio::net::TcpStream> {
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
            Ok(stream) => return Ok(stream),
            Err(_err) if start.elapsed() < std::time::Duration::from_secs(10) => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

async fn send_packet<S>(stream: &mut S, packet: &str) -> anyhow::Result<()>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    let checksum = packet.bytes().fold(0u8, |acc, byte| acc.wrapping_add(byte));
    stream
        .write_all(format!("${packet}#{checksum:02x}").as_bytes())
        .await?;
    stream.flush().await?;
    Ok(())
}

async fn read_packet<S>(stream: &mut S) -> anyhow::Result<String>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut packet = Vec::new();
    let mut in_packet = false;
    loop {
        let byte = stream.read_u8().await?;
        match byte {
            b'+' | b'-' if !in_packet => {}
            b'$' => {
                packet.clear();
                in_packet = true;
            }
            b'#' if in_packet => {
                let _checksum = stream.read_u16().await?;
                // The peer may already have dropped its read half after
                // sending the final packet, so the ack is best-effort.
                let _ = stream.write_all(b"+").await;
                return Ok(String::from_utf8_lossy(&packet).into_owned());
            }
            _ if in_packet => packet.push(byte),
            _ => {}
        }
    }
}
