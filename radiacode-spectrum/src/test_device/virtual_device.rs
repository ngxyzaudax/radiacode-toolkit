use std::collections::{BTreeMap, HashMap, VecDeque};

use radiacode_lib::Error;
use radiacode_lib::protocol::{Command, VirtSfr, VirtString};

pub const DEFAULT_SFR_FILE: &str = "[VSFR_DISP_BRT]\nAddr=0x00000511\nSize=1\nType=1\n[VSFR_TEMP_degC]\nAddr=0x00008024\nSize=4\nType=5\n";
pub const LEDS_SFR_FILE: &str = "[VSFR_DISP_BRT]\nAddr=0x00000511\nSize=1\nType=1\n[VSFR_LEDS_ON]\nAddr=0x00000545\nSize=1\nType=1\n";
pub const DEFAULT_CONFIGURATION: &str = "SpecFormatVersion=1\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRequest {
    pub command: u16,
    pub seq: u8,
    pub args: Vec<u8>,
}

#[derive(Debug)]
pub enum ScriptedReply {
    Payload(Vec<u8>),
    RawResponse(Vec<u8>),
    Failure(Error),
}

impl ScriptedReply {
    pub fn timeout() -> Self {
        Self::Failure(Error::Timeout)
    }

    pub fn connection_closed() -> Self {
        Self::Failure(Error::ConnectionClosed)
    }
}

#[derive(Debug)]
pub struct VirtualDevice {
    pub target_firmware: (u16, u16),
    pub boot_firmware: (u16, u16),
    pub hw_serial_groups: Vec<u32>,
    pub status_word: u32,
    pub registers: BTreeMap<u32, u32>,
    pub strings: HashMap<u32, Vec<u8>>,
    pub batch_supported: bool,
    pub requests: Vec<RecordedRequest>,
    pub scripts: HashMap<u16, VecDeque<ScriptedReply>>,
}

impl Default for VirtualDevice {
    fn default() -> Self {
        let strings = [
            (VirtString::SfrFile, DEFAULT_SFR_FILE.as_bytes()),
            (VirtString::Configuration, DEFAULT_CONFIGURATION.as_bytes()),
            (VirtString::SerialNumber, b"RC-103-001234".as_slice()),
        ]
        .into_iter()
        .map(|(id, data)| (u32::from(id), data.to_vec()))
        .collect();
        Self {
            target_firmware: (4, 12),
            boot_firmware: (4, 0),
            hw_serial_groups: vec![0x0102_0304, 0x0506_0708],
            status_word: 0x1234,
            registers: BTreeMap::new(),
            strings,
            batch_supported: true,
            requests: Vec::new(),
            scripts: HashMap::new(),
        }
    }
}

impl VirtualDevice {
    pub fn with_firmware(mut self, major: u16, minor: u16) -> Self {
        self.target_firmware = (major, minor);
        self
    }

    pub fn with_register(mut self, register: VirtSfr, value: u32) -> Self {
        self.registers.insert(u32::from(register), value);
        self
    }

    pub fn with_string(mut self, id: VirtString, data: &[u8]) -> Self {
        self.strings.insert(u32::from(id), data.to_vec());
        self
    }

    pub fn without_batch(mut self) -> Self {
        self.batch_supported = false;
        self
    }

    pub fn with_script(mut self, command: Command, reply: ScriptedReply) -> Self {
        self.push_script(command, reply);
        self
    }

    pub fn push_script(&mut self, command: Command, reply: ScriptedReply) {
        self.scripts
            .entry(u16::from(command))
            .or_default()
            .push_back(reply);
    }

    pub fn register(&self, register: VirtSfr) -> Option<u32> {
        self.registers.get(&u32::from(register)).copied()
    }

    pub fn requests_for(&self, command: Command) -> Vec<&RecordedRequest> {
        self.requests
            .iter()
            .filter(|request| request.command == u16::from(command))
            .collect()
    }

    pub fn request_count(&self, command: Command) -> usize {
        self.requests_for(command).len()
    }

    pub fn take_script(&mut self, command: u16) -> Option<ScriptedReply> {
        self.scripts.get_mut(&command).and_then(VecDeque::pop_front)
    }
}
