// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Panic handling for the application.

use n2o4::cfe::es;
use n2o4::cfe::es::RunStatus;

/// Whether to restart the application on panic.
/// If false, the application will exit on panic instead.
const RESTART_APP: bool = false;

/// Custom panic handler for this application.
///
/// Instead of aborting the process, this logs a message
/// and tries to exit the application through cFE.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if let Some(l) = info.location() {
        es::write_to_syslog2(
            pfmt!(
                "RUST_SAMPLE: Rust panic occurred at [source file unimplemented], line %u, column %u."
            ),
            l.line(),
            l.column(),
        );
    } else {
        es::write_to_syslog_str("RUST_SAMPLE: Rust panic occurred.");
    }

    es::exit_app(if RESTART_APP { RunStatus::SysRestart } else { RunStatus::AppError });

    // The preceeding function call should not return.
    // If, for some reason, it *does* return:
    loop {}
}
