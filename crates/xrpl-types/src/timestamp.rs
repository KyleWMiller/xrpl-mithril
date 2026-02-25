//! XRPL timestamps.
//!
//! The XRPL uses a custom epoch: January 1, 2000 00:00:00 UTC.
//! All timestamps in the protocol are seconds since this epoch,
//! stored as [`u32`] values.

use serde::{Deserialize, Serialize};

/// Seconds between the Unix epoch (1970-01-01) and the Ripple epoch (2000-01-01).
pub const RIPPLE_EPOCH_OFFSET: u64 = 946_684_800;

/// A timestamp in the XRPL protocol.
///
/// Stored as seconds since the Ripple epoch (2000-01-01T00:00:00Z).
/// This is a `UInt32` in the binary format, giving a range from
/// 2000-01-01 to approximately 2136-02-07.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RippleTimestamp(u32);

impl RippleTimestamp {
    /// Creates a `RippleTimestamp` from seconds since the Ripple epoch.
    #[must_use]
    pub const fn from_ripple_epoch(seconds: u32) -> Self {
        Self(seconds)
    }

    /// Returns the timestamp as seconds since the Ripple epoch.
    #[must_use]
    pub const fn as_ripple_epoch(&self) -> u32 {
        self.0
    }

    /// Converts a Unix timestamp (seconds since 1970-01-01) to a Ripple timestamp.
    ///
    /// Returns `None` if the Unix timestamp is before the Ripple epoch.
    #[must_use]
    pub const fn from_unix(unix_seconds: u64) -> Option<Self> {
        if unix_seconds < RIPPLE_EPOCH_OFFSET {
            return None;
        }
        let ripple_seconds = unix_seconds - RIPPLE_EPOCH_OFFSET;
        if ripple_seconds > u32::MAX as u64 {
            return None;
        }
        Some(Self(ripple_seconds as u32))
    }

    /// Converts to a Unix timestamp (seconds since 1970-01-01).
    #[must_use]
    pub const fn to_unix(&self) -> u64 {
        self.0 as u64 + RIPPLE_EPOCH_OFFSET
    }
}

impl core::fmt::Display for RippleTimestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Display as Ripple epoch seconds; a proper ISO 8601 display
        // would require a datetime library (feature-gated in Phase 4)
        write!(f, "RippleEpoch({})", self.0)
    }
}
