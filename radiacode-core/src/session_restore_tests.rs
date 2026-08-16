use radiacode_protocol::{DeviceVersions, FirmwareVersion};

use crate::session_restore::SessionRestore;

#[test]
fn session_restore_roundtrip_fields() {
    let restore = SessionRestore {
        versions: DeviceVersions {
            boot: FirmwareVersion {
                major: 1,
                minor: 0,
                date: "20240101".into(),
            },
            target: FirmwareVersion {
                major: 2,
                minor: 1,
                date: "20240102".into(),
            },
        },
        spectrum_format_version: 7,
    };
    assert_eq!(restore.versions.target.major, 2);
    assert_eq!(restore.spectrum_format_version, 7);
}
