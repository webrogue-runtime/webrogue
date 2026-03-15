use std::io::Write as _;

use tokio::io::AsyncReadExt as _;

use crate::{
    communication::DebuggerLoopMessage,
    connection::Connection,
    gdb_stub_target::{StopReason, Wasm32Target},
    BoxedPacketReceiver, BoxedPacketSender,
};

pub fn run(
    mut receiver: BoxedPacketReceiver,
    sender: BoxedPacketSender,
    mut target: Wasm32Target,
) -> anyhow::Result<()> {
    let connection = Connection::new(sender);
    let stub = gdbstub::stub::GdbStub::builder(connection).build()?;

    let mut state_machine = stub.run_state_machine(&mut target)?;

    loop {
        use gdbstub::stub::state_machine::GdbStubStateMachine::*;
        match state_machine {
            Idle(mut gdb) => {
                gdb.borrow_conn().flush()?;
                let byte = receive_byte_sync(&mut receiver)?;
                state_machine = gdb.incoming_data(&mut target, byte)?;
            }
            Running(mut gdb) => {
                gdb.borrow_conn().flush()?;
                let select_result = select(&mut target, &mut receiver)?;
                match select_result {
                    SelectResult::Stop(stop_reason) => match stop_reason {
                        StopReason::Paused(reason, registers) => {
                            target.ensure_all_threads_are_paused()?;
                            if let Some(registers) = registers {
                                let pc_bytes = registers.pc.to_le_bytes();
                                let mut regs = core::iter::once((
                                    gdbstub_arch::wasm::reg::id::WasmRegId::Pc,
                                    pc_bytes.as_slice(),
                                ));
                                state_machine =
                                    gdb.report_stop_with_regs(&mut target, reason, &mut regs)?;
                            } else {
                                state_machine = gdb.report_stop(&mut target, reason)?;
                            };
                        }
                        StopReason::Finished => {
                            break;
                        }
                    },
                    SelectResult::IncomingData(byte) => {
                        state_machine = gdb.incoming_data(&mut target, byte)?;
                    }
                }
            }
            CtrlCInterrupt(gdb) => {
                let stop_reason = target.pause_a_thread()?;
                match stop_reason {
                    StopReason::Paused(stop_reason, _registers) => {
                        target.ensure_all_threads_are_paused()?;
                        state_machine = gdb.interrupt_handled(&mut target, Some(stop_reason))?;
                    }
                    StopReason::Finished => break,
                };
            }
            Disconnected(_gdb) => break,
        }
    }
    Ok(())
}

async fn receive_byte(receiver: &mut BoxedPacketReceiver) -> anyhow::Result<u8> {
    let byte = receiver.read_u8().await?;
    eprint!("{}", byte as char);
    std::io::stderr().flush().unwrap();
    Ok(byte)
}

fn receive_byte_sync(receiver: &mut BoxedPacketReceiver) -> anyhow::Result<u8> {
    tokio::runtime::Handle::current().block_on(async { receive_byte(receiver).await })
}

enum SelectResult {
    Stop(StopReason),
    IncomingData(u8),
}

fn select(
    target: &mut Wasm32Target,
    receiver: &mut BoxedPacketReceiver,
) -> anyhow::Result<SelectResult> {
    Ok(tokio::runtime::Handle::current().block_on(async {
        tokio::select! {
            reason = target.wain_for_a_stop() => anyhow::Ok(SelectResult::Stop(reason?)),
            byte = receive_byte(receiver) => anyhow::Ok(SelectResult::IncomingData(byte?)),
        }
    })?)
}
