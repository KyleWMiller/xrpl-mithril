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
///
/// # Examples
///
/// Creating from the Ripple epoch:
///
/// ```
/// use xrpl_types::RippleTimestamp;
///
/// let ts = RippleTimestamp::from_ripple_epoch(0);
/// // Ripple epoch 0 = Unix epoch 946_684_800 (2000-01-01T00:00:00Z)
/// assert_eq!(ts.to_unix(), 946_684_800);
/// ```
///
/// Converting from a Unix timestamp:
///
/// ```
/// use xrpl_types::RippleTimestamp;
///
/// // Unix timestamp for 2025-01-01T00:00:00Z
/// let ts = RippleTimestamp::from_unix(1_735_689_600).unwrap();
/// assert_eq!(ts.to_unix(), 1_735_689_600);
///
/// // Timestamps before the Ripple epoch return None:
/// assert!(RippleTimestamp::from_unix(0).is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RippleTimestamp(u32);

impl RippleTimestamp {
    /// Creates a `RippleTimestamp` from seconds since the Ripple epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::RippleTimestamp;
    ///
    /// let ts = RippleTimestamp::from_ripple_epoch(789_004_800);
    /// assert_eq!(ts.as_ripple_epoch(), 789_004_800);
    /// ```
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
    /// Returns `None` if the Unix timestamp is before the Ripple epoch
    /// (946,684,800 = 2000-01-01T00:00:00Z) or if it overflows `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::RippleTimestamp;
    ///
    /// let ts = RippleTimestamp::from_unix(946_684_800).unwrap();
    /// assert_eq!(ts.as_ripple_epoch(), 0); // Ripple epoch start
    ///
    /// // Before the Ripple epoch:
    /// assert!(RippleTimestamp::from_unix(946_684_799).is_none());
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::RippleTimestamp;
    ///
    /// let ts = RippleTimestamp::from_ripple_epoch(0);
    /// assert_eq!(ts.to_unix(), 946_684_800);
    /// ```
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
