use radiacode_protocol::DeviceVersions;

#[derive(Debug, Clone, PartialEq)]
pub struct SessionRestore {
    pub versions: DeviceVersions,
    pub spectrum_format_version: u32,
}
