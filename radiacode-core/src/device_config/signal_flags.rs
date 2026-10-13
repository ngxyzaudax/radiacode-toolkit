const CTRL_BUTTONS: u32 = 1 << 0;
const CTRL_CLICKS: u32 = 1 << 1;
const CTRL_DR_ALARM1: u32 = 1 << 2;
const CTRL_DR_ALARM2: u32 = 1 << 3;
const CTRL_DR_OOS: u32 = 1 << 4;
const CTRL_DOSE_ALARM1: u32 = 1 << 5;
const CTRL_DOSE_ALARM2: u32 = 1 << 6;
const CTRL_DOSE_OOS: u32 = 1 << 7;
const CTRL_CONNECTION: u32 = 1 << 8;
const CTRL_POWER: u32 = 1 << 9;
const CTRL_CR_ALARM1: u32 = 1 << 10;
const CTRL_CR_ALARM2: u32 = 1 << 11;
const CTRL_CR_OOS: u32 = 1 << 12;
const CTRL_KNOWN_MASK: u32 = 0x1FFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalFlags {
    pub buttons: bool,
    pub clicks: bool,
    pub dose_rate_alarm1: bool,
    pub dose_rate_alarm2: bool,
    pub dose_rate_out_of_scale: bool,
    pub dose_alarm1: bool,
    pub dose_alarm2: bool,
    pub dose_out_of_scale: bool,
    pub connection: bool,
    pub power: bool,
    pub count_rate_alarm1: bool,
    pub count_rate_alarm2: bool,
    pub count_rate_out_of_scale: bool,
    pub reserved: u32,
}

impl SignalFlags {
    pub fn from_raw(raw: u32) -> Self {
        let bit = |mask: u32| raw & mask != 0;
        Self {
            buttons: bit(CTRL_BUTTONS),
            clicks: bit(CTRL_CLICKS),
            dose_rate_alarm1: bit(CTRL_DR_ALARM1),
            dose_rate_alarm2: bit(CTRL_DR_ALARM2),
            dose_rate_out_of_scale: bit(CTRL_DR_OOS),
            dose_alarm1: bit(CTRL_DOSE_ALARM1),
            dose_alarm2: bit(CTRL_DOSE_ALARM2),
            dose_out_of_scale: bit(CTRL_DOSE_OOS),
            connection: bit(CTRL_CONNECTION),
            power: bit(CTRL_POWER),
            count_rate_alarm1: bit(CTRL_CR_ALARM1),
            count_rate_alarm2: bit(CTRL_CR_ALARM2),
            count_rate_out_of_scale: bit(CTRL_CR_OOS),
            reserved: raw & !CTRL_KNOWN_MASK,
        }
    }

    pub fn as_raw(self) -> u32 {
        self.bit_table()
            .into_iter()
            .filter(|(enabled, _)| *enabled)
            .fold(self.reserved, |raw, (_, mask)| raw | mask)
    }

    pub fn vibro_events(self) -> Self {
        Self {
            clicks: false,
            connection: false,
            power: false,
            ..self
        }
    }

    fn bit_table(self) -> [(bool, u32); 13] {
        [
            (self.buttons, CTRL_BUTTONS),
            (self.clicks, CTRL_CLICKS),
            (self.dose_rate_alarm1, CTRL_DR_ALARM1),
            (self.dose_rate_alarm2, CTRL_DR_ALARM2),
            (self.dose_rate_out_of_scale, CTRL_DR_OOS),
            (self.dose_alarm1, CTRL_DOSE_ALARM1),
            (self.dose_alarm2, CTRL_DOSE_ALARM2),
            (self.dose_out_of_scale, CTRL_DOSE_OOS),
            (self.connection, CTRL_CONNECTION),
            (self.power, CTRL_POWER),
            (self.count_rate_alarm1, CTRL_CR_ALARM1),
            (self.count_rate_alarm2, CTRL_CR_ALARM2),
            (self.count_rate_out_of_scale, CTRL_CR_OOS),
        ]
    }
}
