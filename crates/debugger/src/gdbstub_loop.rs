use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt as _};

use crate::{
    communication::RunnerMessage,
    connection::{Connection, PacketSender},
    target::Wasm32Target,
};

pub fn gdbstub_loop(
    message_rx: tokio::sync::mpsc::Receiver<RunnerMessage>,
    mut gdb_rx: Pin<Box<dyn AsyncRead + Send>>,
    gdb_tx: Box<dyn PacketSender + Send>,
    rt: tokio::runtime::Handle,
) -> anyhow::Result<()> {
    let connection = Connection::new(gdb_tx);
    let stub = gdbstub::stub::GdbStub::builder(connection).build()?;
    let mut target = Wasm32Target::new(message_rx, rt.clone());
    let mut state_machine = stub.run_state_machine(&mut target)?;
    rt.block_on(target.wait_for_a_pause());
    if target.is_finished() {
        return Ok(());
    }

    loop {
        use gdbstub::stub::state_machine::GdbStubStateMachine::*;
        match state_machine {
            Idle(mut gdb) => {
                let byte = rt.block_on(async {
                    gdb.borrow_conn().flush().await?;
                    let byte = gdb_rx.read_u8().await?;
                    anyhow::Ok(byte)
                })?;
                state_machine = gdb.incoming_data(&mut target, byte)?;
            }
            Running(mut gdb) => {
                let select_result = rt.block_on(async {
                    gdb.borrow_conn().flush().await?;

                    let select_result = tokio::select! {
                        byte = gdb_rx.read_u8() => SelectResult::Data(byte?),
                        _ = target.wait_for_a_pause() => SelectResult::Paused
                    };
                    anyhow::Ok(select_result)
                })?;

                match select_result {
                    SelectResult::Data(byte) => {
                        state_machine = gdb.incoming_data(&mut target, byte)?;
                    }
                    SelectResult::Paused => {
                        let (reason, registers) = target.get_pause_reason()?;
                        if let Some(registers) = registers {
                            let pc_bytes = registers.pc.to_le_bytes();
                            let mut regs = core::iter::once((
                                gdbstub_arch::wasm::reg::id::WasmRegId::Pc,
                                pc_bytes.as_slice(),
                            ));

                            state_machine =
                                gdb.report_stop_with_regs(&mut target, reason, &mut regs)?
                        } else {
                            state_machine = gdb.report_stop(&mut target, reason)?
                        };
                    }
                }
            }
            CtrlCInterrupt(gdb) => {
                target.interrupt();
                rt.block_on(target.wait_for_a_pause());
                let (reason, _) = target.get_pause_reason()?;
                state_machine = gdb.interrupt_handled(&mut target, Some(reason))?;
            }
            Disconnected(mut gdb) => {
                rt.block_on(gdb.borrow_conn().flush())?;
                break;
            }
        }
    }
    anyhow::Ok(())
}

enum SelectResult {
    Data(u8),
    Paused,
}
