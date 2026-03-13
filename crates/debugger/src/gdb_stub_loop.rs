use std::io::Write as _;

use tokio::io::AsyncReadExt as _;

use crate::{
    communication::DebuggerLoopMessage,
    connection::Connection,
    gdb_stub_target::{HandleMessageResult, Wasm32Target},
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
                let byte = receive_byte(&mut receiver)?;
                std::io::stderr().flush().unwrap();
                state_machine = gdb.incoming_data(&mut target, byte)?;
            }
            Running(mut gdb) => {
                gdb.borrow_conn().flush()?;
                let select_result = receive_message_or_byte(&mut target, &mut receiver)?;
                match select_result {
                    SelectResult::Message(message) => match target.handle_message(message)? {
                        Some(HandleMessageResult::CodeStopped(reason, registers)) => {
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
                        Some(HandleMessageResult::Finished) => {
                            break;
                        }
                        None => state_machine = gdb.into(),
                    },
                    SelectResult::IncomingData(byte) => {
                        state_machine = gdb.incoming_data(&mut target, byte)?;
                    }
                }
            }
            CtrlCInterrupt(gdb) => {
                let (reason, _registers) = target.interrupt_all_thread()?;
                state_machine = gdb.interrupt_handled(&mut target, Some(reason))?;
            }
            Disconnected(_gdb) => break,
        }
    }
    Ok(())
}

fn receive_byte(receiver: &mut BoxedPacketReceiver) -> anyhow::Result<u8> {
    let byte = tokio::runtime::Handle::current().block_on(async { receiver.read_u8().await })?;
    eprint!("{}", byte as char);
    Ok(byte)
}

enum SelectResult {
    Message(DebuggerLoopMessage),
    IncomingData(u8),
}

fn receive_message_or_byte(
    target: &mut Wasm32Target,
    receiver: &mut BoxedPacketReceiver,
) -> anyhow::Result<SelectResult> {
    Ok(tokio::runtime::Handle::current().block_on(async {
        tokio::select! {
            message =  target.receive_message() => anyhow::Ok(SelectResult::Message(message?)),
            byte = receiver.read_u8() => anyhow::Ok(SelectResult::IncomingData(byte?)),
        }
    })?)
}
