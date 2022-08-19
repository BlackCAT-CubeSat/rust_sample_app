// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Constants for rust_sample.

use cfs::cfe::evs::bin_filter as filt;
use cfs::cfe::evs::BinFilter;
use cfs::cfe::msg::FunctionCode;
use cfs::cfe::sb::MsgId_Atom;

// Event IDs:

/// Event ID: reserved.
pub const RESERVED_EID: u16 = 0;

/// Event ID:
pub const STARTUP_INF_EID: u16 = 1;

/// Event ID:
pub const COMMAND_ERR_EID: u16 = 2;

/// Event ID:
pub const COMMANDNOP_INF_EID: u16 = 3;

/// Event ID:
pub const COMMANDRST_INF_EID: u16 = 4;

/// Event ID:
pub const INVALID_MSGID_ERR_EID: u16 = 5;

/// Event ID:
pub const LEN_ERR_EID: u16 = 6;

/// Event ID:
pub const PIPE_ERR_EID: u16 = 7;

/*
/// Event ID: failure to initialize.
pub const INIT_FAIL_EID: u16 = 1;

/// Event ID: successfully initialized.
pub const INIT_SUCCEED_EID: u16 = 2;

/// Event ID: error receiving message.
pub const MSG_RCV_FAIL_EID: u16 = 3;

/// Event ID: message with invalid format (unknown MID, wrong size) received.
pub const INVALID_MSG_EID: u16 = 4;

/// Event ID: error sending message.
pub const MSG_TRANSMIT_FAIL_EID: u16 = 5;
*/

/// The set of event filters for the application.
#[rustfmt::skip]
pub const EVENT_FILTERS: &[BinFilter] = &[
    BinFilter { EventID: STARTUP_INF_EID,       Mask: filt::NO_FILTER },
    BinFilter { EventID: COMMAND_ERR_EID,       Mask: filt::NO_FILTER },
    BinFilter { EventID: COMMANDNOP_INF_EID,    Mask: filt::NO_FILTER },
    BinFilter { EventID: COMMANDRST_INF_EID,    Mask: filt::NO_FILTER },
    BinFilter { EventID: INVALID_MSGID_ERR_EID, Mask: filt::NO_FILTER },
    BinFilter { EventID: LEN_ERR_EID,           Mask: filt::NO_FILTER },
    BinFilter { EventID: PIPE_ERR_EID,          Mask: filt::NO_FILTER },
];

// Performance log IDs (these should be unique across all apps on the CPU -- change these as needed):

/// Application is running.
pub const RUST_SAMPLE_APP_PERF_ID: u32 = 91;

// Message IDs (these should be unique across all apps on the CPU):

/// Message ID for commands to this application.
pub const CMD_MID: MsgId_Atom = 0x1882;

/// Function code for CMD_MID: no-op.
pub const NOOP_CC: FunctionCode = 0;

/// Function code for CMD_MID: reset the counters.
pub const RESET_COUNTERS_CC: FunctionCode = 1;

/// Function code for CMD_MID: do processing.
pub const PROCESS_CC: FunctionCode = 2;

/// Message ID to make this application emit telemetry.
pub const SEND_HK_MID: MsgId_Atom = 0x1883;

/// Message ID for this application's telemetry.
pub const HK_TLM_MID: MsgId_Atom = 0x0883;
