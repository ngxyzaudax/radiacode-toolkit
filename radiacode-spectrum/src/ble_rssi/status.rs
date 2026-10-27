use radiacode_lib::RssiStatus;

/// Result of asking the BlueZ management socket for the connected link's RSSI.
///
/// That socket requires `CAP_NET_ADMIN`. The `bluetooth` group is enough for
/// `bluetoothctl` over D-Bus and is not enough for this query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MgmtRssi {
    /// RSSI in dBm.
    Measured(i16),
    /// The kernel refused the management-socket query.
    PermissionDenied,
    /// No management-socket sample is available.
    Unavailable,
}

/// Prefers a management-socket measurement, then a cached measurement.
///
/// A permission denial is kept when the cache has no measurement. Other
/// management failures fall back to a cached measurement or [`RssiStatus::NotReported`].
pub fn combine_rssi(mgmt: MgmtRssi, cached: RssiStatus) -> RssiStatus {
    match (mgmt, cached) {
        (MgmtRssi::Measured(value), _) => RssiStatus::Measured(value),
        (MgmtRssi::PermissionDenied, RssiStatus::Measured(value)) => RssiStatus::Measured(value),
        (MgmtRssi::PermissionDenied, _) => RssiStatus::PermissionDenied,
        (MgmtRssi::Unavailable, RssiStatus::Measured(value)) => RssiStatus::Measured(value),
        (MgmtRssi::Unavailable, _) => RssiStatus::NotReported,
    }
}

#[cfg(test)]
mod tests {
    use super::{MgmtRssi, combine_rssi};
    use radiacode_lib::RssiStatus;

    #[test]
    fn measured_mgmt_rssi_wins_over_the_cache() {
        let combined = combine_rssi(MgmtRssi::Measured(-62), RssiStatus::Measured(-70));
        assert_eq!(combined, RssiStatus::Measured(-62));
    }

    #[test]
    fn permission_denied_keeps_a_cached_measurement() {
        let combined = combine_rssi(MgmtRssi::PermissionDenied, RssiStatus::Measured(-70));
        assert_eq!(combined, RssiStatus::Measured(-70));
    }

    #[test]
    fn permission_denied_is_visible_when_nothing_was_cached() {
        let combined = combine_rssi(MgmtRssi::PermissionDenied, RssiStatus::NotReported);
        assert_eq!(combined, RssiStatus::PermissionDenied);
        assert_eq!(combined.to_string(), "RSSI unavailable: permission denied");
    }

    #[test]
    fn other_mgmt_failures_fall_back_without_hiding_a_denial() {
        assert_eq!(
            combine_rssi(MgmtRssi::Unavailable, RssiStatus::NotReported),
            RssiStatus::NotReported
        );
        assert_eq!(
            combine_rssi(MgmtRssi::Unavailable, RssiStatus::Measured(-55)),
            RssiStatus::Measured(-55)
        );
    }
}
