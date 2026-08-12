//! How many of this machine's cores are parked.
//!
//! **Why this exists.** `machine_cpu_percent` says what the box delivered and
//! cannot say what it was willing to deliver. On 2026-08-12 this laptop began
//! parking cores after a Windows update whose notes state that "sleep, display,
//! and power setting changes now apply correctly across all power plans",
//! switching on a vendor scheme's previously-inert values. Six quiet
//! hundred-session holds gave **16.11 and 16.00 machine cores on 3 and 11
//! August against 4.52-5.37 on the 12th**, and no artifact could distinguish
//! the two machines: same build string, same scheme name, same everything the
//! host block recorded.
//!
//! **Why WMI rather than PDH.** The counter is
//! `Win32_PerfFormattedData_Counters_ProcessorInformation` -> `ParkingStatus`,
//! and PDH needs `unsafe`, which the workspace forbids at its root. Reached
//! through a crate for the same reason `win32job` is, and checked before it was
//! chosen: the class exposes the property here, and WMI read 12 of 16 parked
//! against the perf counter's 11 seconds apart — agreement for a count measured
//! to swing 0 to 12 within seconds.
//!
//! **It costs about 300 ms a tick, measured as a matched pair.** Two 150-second
//! holds in the same machine state, 28 intervals each, slipped **+353.0 and
//! +347.5 ms** against a pre-WMI baseline of +35 to +64 ms in the same regime —
//! agreement to 1.6%, so this is a measurement rather than the single-hold hint
//! that prompted it. That is **6% of a 5-second tick and five to ten times the
//! whole prior sampler cost**, spent to distinguish cores taken elsewhere from
//! cores switched off. Worth it at 5 s; **at the 1 s interval the census
//! scripts use it would be a third of the budget**, and the rule this
//! repository carries is that the observer becoming the bottleneck is the one
//! failure that does not announce itself.
//!
//! **Read the distribution, never a mean.** Idle, this count is bimodal: 0 in
//! 39% of samples and 12 in 27%. A mean over it describes no state the machine
//! is ever in.

use serde::Deserialize;
use wmi::WMIConnection;

#[derive(Deserialize)]
#[serde(rename = "Win32_PerfFormattedData_Counters_ProcessorInformation")]
#[serde(rename_all = "PascalCase")]
struct ProcessorInformation {
    name: String,
    parking_status: u32,
}

/// A live WMI connection, or the reason there is not one.
pub struct Parking {
    connection: Option<WMIConnection>,
    /// Kept so an unavailable counter reads as an absence rather than as zero
    /// parked cores, which is the state a healthy machine is in.
    pub unavailable: Option<String>,
}

impl Parking {
    pub fn connect() -> Self {
        // `WMIConnection::new` initialises COM itself in 0.18; an earlier
        // reading of this API assumed a separate `COMLibrary` handle, which the
        // crate no longer exports.
        match WMIConnection::new() {
            Ok(connection) => Self {
                connection: Some(connection),
                unavailable: None,
            },
            Err(err) => Self {
                connection: None,
                unavailable: Some(err.to_string()),
            },
        }
    }

    /// Cores currently parked, or `None` when the counter cannot be read.
    ///
    /// `None` rather than `0`: an unreadable counter and a fully unparked
    /// machine are opposite facts, and a zero would make the first read as the
    /// second on every artifact that stores it.
    pub fn parked_cores(&self) -> Option<u32> {
        let connection = self.connection.as_ref()?;
        let rows: Vec<ProcessorInformation> = connection.query().ok()?;
        let parked = rows
            .iter()
            .filter(|r| !r.name.contains("_Total"))
            .filter(|r| r.parking_status > 0)
            .count();
        Some(parked as u32)
    }
}
