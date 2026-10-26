use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crate::test_device::encode::{rare_record, real_time_record, spectrum_v0_payload};
use crate::test_device::{DeviceHandle, ScriptedReply, VirtualDevice, open_device};
use crossbeam_channel::{Receiver, Sender, unbounded};
use radiacode_lib::config::{
    AlarmLimits, AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection, LightControl,
    SignalFlags,
};
use radiacode_lib::protocol::{Command, VirtSfr, VirtString};
use radiacode_lib::{CountDisplayUnit, DeviceEndpoint, DoseDisplayUnit};

use super::epoch::SessionEpoch;
use super::messages::WorkerEvent;
use super::session::WorkerSession;

pub fn alarm_limits() -> AlarmLimits {
    AlarmLimits {
        l1_count_rate: 30.0,
        l2_count_rate: 60.0,
        l1_dose_rate: 1.0,
        l2_dose_rate: 2.0,
        l1_dose: 1000.0,
        l2_dose: 2000.0,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        count_unit: CountDisplayUnit::Cps,
    }
}

pub fn device_config() -> DeviceConfig {
    DeviceConfig {
        alarms: alarm_limits(),
        alarm_mode: AlarmSignalMode::Once,
        brightness: 5,
        backlight_off: BacklightOffTime::Sec5,
        display_dir: DisplayDirection::Auto,
        sound_on: true,
        vibro_on: false,
        leds_on: false,
        light_control: LightControl::DeviceCtrlBit,
        sound_ctrl: SignalFlags::from_raw(0x1f9d),
        vibro_ctrl: SignalFlags::from_raw(0x1fff),
    }
}

pub fn usb_endpoint() -> DeviceEndpoint {
    DeviceEndpoint::Usb {
        serial: "RC-103-001234".to_string(),
    }
}

pub fn monitor_frame() -> Vec<u8> {
    let mut bytes = real_time_record(1, 100, 5.0, 0.0001);
    bytes.extend(rare_record(2, 200, 87.5, 24.5));
    bytes
}

pub fn configured_device() -> VirtualDevice {
    VirtualDevice::default()
        .with_register(VirtSfr::CrLev1Cp10s, 300)
        .with_register(VirtSfr::CrLev2Cp10s, 600)
        .with_register(VirtSfr::DrLev1UrH, 100)
        .with_register(VirtSfr::DrLev2UrH, 200)
        .with_register(VirtSfr::DsLev1Ur, 1000)
        .with_register(VirtSfr::DsLev2Ur, 2000)
        .with_register(VirtSfr::DsUnits, 1)
        .with_register(VirtSfr::CrUnits, 0)
        .with_register(VirtSfr::AlarmMode, 1)
        .with_register(VirtSfr::DispBrt, 5)
        .with_register(VirtSfr::DispOffTime, 3)
        .with_register(VirtSfr::DispDir, 2)
        .with_register(VirtSfr::SoundOn, 1)
        .with_register(VirtSfr::SoundCtrl, 0x1f9d)
        .with_register(VirtSfr::VibroOn, 0)
        .with_register(VirtSfr::VibroCtrl, 0x1fff)
        .with_register(VirtSfr::DeviceCtrl, 0x35)
        .with_register(VirtSfr::TempDegC, 31.0f32.to_bits())
        .with_string(VirtString::Configuration, b"SpecFormatVersion=0\n")
        .with_string(VirtString::DataBuf, &monitor_frame())
        .with_string(
            VirtString::Spectrum,
            &spectrum_v0_payload(120, [1.0, 2.0, 3.0], &[5, 6, 7]),
        )
}

pub struct Harness {
    pub session: WorkerSession,
    pub events: Receiver<WorkerEvent>,
    pub sender: Sender<WorkerEvent>,
    pub handle: DeviceHandle,
    pub live_epoch: Arc<AtomicU64>,
}

impl Harness {
    pub async fn connected(device: VirtualDevice) -> Self {
        let (radiacode, handle) = open_device(device).await;
        let (sender, events) = unbounded();
        let mut session = WorkerSession::new();
        session.session_restore = radiacode.session_restore();
        session.device = Some(radiacode);
        Self {
            session,
            events,
            sender,
            handle,
            live_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn script(&self, command: Command, reply: ScriptedReply) {
        self.handle.device().push_script(command, reply);
    }

    pub fn drain(&self) -> Vec<WorkerEvent> {
        self.events.try_iter().collect()
    }
}

pub fn epoch_of(live_epoch: &Arc<AtomicU64>) -> SessionEpoch {
    SessionEpoch::capture(live_epoch)
}

pub fn count_events(events: &[WorkerEvent], matcher: fn(&WorkerEvent) -> bool) -> usize {
    events.iter().filter(|event| matcher(event)).count()
}

pub fn is_monitor_sample(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::MonitorSample(_))
}

pub fn is_spectrum(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Spectrum(_))
}

pub fn is_error(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Error(_))
}

pub fn is_disconnected(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Disconnected)
}
