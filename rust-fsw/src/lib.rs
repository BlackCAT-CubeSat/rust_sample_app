// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! The Rust_sample cFS application.

#![cfg_attr(not(test), no_std)]
#![feature(inline_const)]

extern crate n2o4;
extern crate printf_wrap;

use core::num::Wrapping;
use n2o4::cfe::evs::EventType as EvT;
use n2o4::cfe::msg::{Command, Message, Telemetry};
use n2o4::cfe::{es, evs, sb, Status};
use printf_wrap::{null_str, NullString};

use crate::constants::*;

/// Shorthand for creating a `const `[`PrintfFmt`](printf_wrap::PrintfFmt)`<(...)>`.
///
/// Note that the type parameters must be deducible from the context.
macro_rules! pfmt {
    ($str:expr) => {
        const { printf_wrap::PrintfFmt::new_or_panic(concat!($str, "\0")) }
    };
}

mod constants;

#[cfg(not(test))]
mod panic;

/// Queue depth for the application's message pipe.
const PIPE_DEPTH: u16 = 32;
/// System name for the application's message pipe.
const PIPE_NAME: NullString = null_str!("RUST_SAMPLE_PIPE");

/// The application's state.
struct AppState {
    /// Command interface counter: number of received commands.
    cmd_counter: Wrapping<u8>,

    /// Command interface counter: number of errors.
    err_counter: Wrapping<u8>,

    /// Marker for sending EVS events.
    ev: evs::EventSender,

    /// Housekeeping telemetry packet.
    tlm: Telemetry<SampleAppHkTlm>,
}

/// The payload of the application housekeeping telemetry packet.
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct SampleAppHkTlm {
    pub command_error_counter: u8,
    pub command_counter: u8,
    pub spare: [u8; 2],
}

/// The application entry point.
#[no_mangle]
pub extern "C" fn SAMPLE_APP_Main() {
    es::perf_log_entry(RUST_SAMPLE_APP_PERF_ID);
    let _ = app();
    es::perf_log_exit(RUST_SAMPLE_APP_PERF_ID);
    es::exit_app(es::RunStatus::AppExit);
}

/// The possibly-fallible parts of [`SAMPLE_APP_Main`].
fn app() -> Result<(), Status> {
    let (mut state, mut pipe) = initialize()?;

    let mut keep_running = true;

    // The application's main event loop:
    while keep_running && es::run_loop(Some(es::RunStatus::AppRun)) {
        es::perf_log_exit(RUST_SAMPLE_APP_PERF_ID);

        let _: Result<(), Status> = pipe.receive_buffer(sb::TimeOut::PendForever, |buf_maybe| {
            es::perf_log_entry(RUST_SAMPLE_APP_PERF_ID);
            let buf = buf_maybe.map_err(|status| {
                state.ev.send_event_str(
                    PIPE_ERR_EID,
                    EvT::Error,
                    "RUST_SAMPLE: SB pipe read error, app will exit",
                );
                keep_running = false;
                status
            })?;

            let _ = process_message(&mut state, buf);
            Ok(())
        });
    }

    Ok(())
}

/// Initalizes application state.
fn initialize() -> Result<(AppState, sb::Pipe), Status> {
    // Set ourselves up with EVS:
    let ev = evs::register(EVENT_FILTERS).map_err(|status| {
        es::write_to_syslog1(
            pfmt!("RUST_SAMPLE: error registering events, code = 0x%08x\n"),
            status.as_num(),
        );
        status
    })?;

    // Create our message pipe:
    let mut pipe = sb::Pipe::new(PIPE_DEPTH, PIPE_NAME).map_err(|status| {
        ev.send_event1(
            STARTUP_INF_EID,
            EvT::Error,
            pfmt!("RUST_SAMPLE: error creating pipe, code = 0x%08x"),
            status.as_num(),
        );
        status
    })?;

    // Subscribe to command packets:
    for mid in [CMD_MID, SEND_HK_MID] {
        pipe.subscribe(mid.into()).map_err(|status| {
            ev.send_event2(
                STARTUP_INF_EID,
                EvT::Critical,
                pfmt!("RUST_SAMPLE: error subscribing to packet 0x%04x, code = 0x%08x"),
                mid,
                status.as_num(),
            );
            status
        })?;
    }

    // Initialize our telemetry packet:
    let tlm = Telemetry::<SampleAppHkTlm>::new_default(HK_TLM_MID.into()).map_err(|status| {
        ev.send_event1(
            STARTUP_INF_EID,
            EvT::Critical,
            pfmt!("RUST_SAMPLE: error initializing housekeeping packet, code = 0x%08x"),
            status.as_num(),
        );
        status
    })?;

    let state = AppState {
        cmd_counter: Wrapping(0),
        err_counter: Wrapping(0),
        ev,
        tlm,
    };

    state.ev.send_event_str(STARTUP_INF_EID, EvT::Information, "RUST_SAMPLE: app initialized.");

    Ok((state, pipe))
}

/// Having received a message, figures out what to do with it.
fn process_message(state: &mut AppState, msg: &Message) -> Result<(), Status> {
    let msgid: Result<sb::MsgId_Atom, Status> = msg.msgid().map(|m| m.into());

    match msgid {
        Err(status) => {
            state.ev.send_event1(
                INVALID_MSGID_ERR_EID,
                EvT::Error,
                pfmt!("RUST_SAMPLE: unable to get message ID, error code = 0x%08x"),
                status.as_num(),
            );
        }
        // Command message:
        Ok(CMD_MID) => {
            process_command(state, msg)?;
        }
        // Request to emit telemetry:
        Ok(SEND_HK_MID) => {
            report_housekeeping(state, msg)?;
        }
        // Unknown message:
        Ok(msgid) => {
            state.ev.send_event1(
                INVALID_MSGID_ERR_EID,
                EvT::Error,
                pfmt!("RUST_SAMPLE: unknown message ID 0x%04x received"),
                msgid as u32,
            );
        }
    }
    Ok(())
}

/// Handles a command ([`RUST_SAMPLE_CMD_MID`]) sent to the application.
fn process_command(state: &mut AppState, msg: &Message) -> Result<(), Status> {
    match msg.fcn_code() {
        Err(status) => {
            state.ev.send_event1(
                COMMAND_ERR_EID,
                EvT::Error,
                pfmt!("RUST_SAMPLE: unable to get function code: err = 0x%08x"),
                status.as_num(),
            );
        }
        Ok(NOOP_CC) => {
            let cmd = verify_cmd_pkt(state, msg)?;
            noop(state, cmd)?;
        }
        Ok(RESET_COUNTERS_CC) => {
            let cmd = verify_cmd_pkt(state, msg)?;
            reset_counters(state, cmd)?;
        }
        Ok(PROCESS_CC) => {
            let cmd = verify_cmd_pkt(state, msg)?;
            process(state, cmd)?;
        }
        Ok(cc) => {
            state.ev.send_event1(
                COMMAND_ERR_EID,
                EvT::Error,
                pfmt!("RUST_SAMPLE: invalid ground command code: CC = %u"),
                cc as u32,
            );
        }
    }
    Ok(())
}

/// Handles a task telemetry request from the housekeeping task.
///
/// This function gathers the app's telemetry, packetize it,
/// and send it over the software bus to the housekeeping task.
fn report_housekeeping(state: &mut AppState, _msg: &Message) -> Result<(), Status> {
    let t = &mut state.tlm.payload;
    (t.command_counter, t.command_error_counter) = (state.cmd_counter.0, state.err_counter.0);

    let _ = state.tlm.time_stamp();
    let _ = state.tlm.transmit(true);

    Ok(())
}

/// Processes a "no-op" command.
fn noop(state: &mut AppState, _cmd: &Command<()>) -> Result<(), Status> {
    state.cmd_counter += 1;

    state.ev.send_event_str(COMMANDNOP_INF_EID, EvT::Information, "RUST_SAMPLE: NOOP command");

    Ok(())
}

/// Processes a "reset counters" command.
fn reset_counters(state: &mut AppState, _cmd: &Command<()>) -> Result<(), Status> {
    (state.cmd_counter, state.err_counter) = (Wrapping(0), Wrapping(0));

    state.ev.send_event_str(COMMANDRST_INF_EID, EvT::Information, "RUST_SAMPLE: RESET command");

    Ok(())
}

/// Processes a "process ground statement" command.
fn process(_state: &mut AppState, _cmd: &Command<()>) -> Result<(), Status> {
    // TODO: implement logic

    Ok(())
}

/// Tries to cast a [`Message`] to a [`Command`] with appropriate payload.
fn verify_cmd_pkt<'a, T: Copy>(
    state: &mut AppState,
    msg: &'a Message,
) -> Result<&'a Command<T>, Status> {
    msg.try_cast_cmd().map_err(|status| {
        let msgid: sb::MsgId_Atom = msg.msgid().map(|id| id.into()).unwrap_or(0);

        state.ev.send_event4(
            LEN_ERR_EID,
            EvT::Error,
            pfmt!("RUST_SAMPLE: invalid msg length: ID = 0x%04x, CC = %u, len = %u, expected = %u"),
            msgid as u32,
            msg.fcn_code().unwrap_or(9999) as u32,
            msg.size().unwrap_or(999_999) as u32,
            core::mem::size_of::<Command<T>>() as u32,
        );
        state.err_counter += 1;

        status
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn sample_test() {
        assert_eq!(2 + 2, 4);
    }
}
