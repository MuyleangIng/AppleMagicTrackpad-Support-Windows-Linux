#![cfg_attr(not(feature = "std"), no_std)]

pub const REPORT_ID_MOUSE: u8 = 0x02;
pub const REPORT_ID_MULTITOUCH: u8 = 0x31;
pub const PTP_MAX_CONTACT_POINTS: usize = 5;
pub const MT2_REPORT_HEADER_SIZE: usize = 4;
pub const MT2_FINGER_SIZE: usize = 9;
pub const MOUSE_REPORT_SIZE: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    Empty,
    MalformedLength,
    UnsupportedReport(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisRange {
    pub min: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParserSettings {
    pub button_disabled: bool,
    pub ignore_near_fingers: bool,
    pub palm_rejection: bool,
}

impl Default for ParserSettings {
    fn default() -> Self {
        Self {
            button_disabled: false,
            ignore_near_fingers: true,
            palm_rejection: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParserConfig {
    pub x: AxisRange,
    pub y: AxisRange,
    pub settings: ParserSettings,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            x: AxisRange { min: 0 },
            y: AxisRange { min: 0 },
            settings: ParserSettings::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mt2Finger {
    pub absolute_x: i16,
    pub absolute_y: i16,
    pub finger: u8,
    pub state: u8,
    pub touch_major: u8,
    pub touch_minor: u8,
    pub size: u8,
    pub pressure: u8,
    pub id: u8,
    pub orientation: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PtpContact {
    pub confidence: bool,
    pub tip_switch: bool,
    pub contact_id: u32,
    pub x: u16,
    pub y: u16,
}

impl Default for PtpContact {
    fn default() -> Self {
        Self {
            confidence: false,
            tip_switch: false,
            contact_id: 0,
            x: 0,
            y: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PtpReport {
    pub report_id: u8,
    pub scan_time: u16,
    pub contact_count: u8,
    pub is_button_clicked: bool,
    pub contacts: [PtpContact; PTP_MAX_CONTACT_POINTS],
}

impl Default for PtpReport {
    fn default() -> Self {
        Self {
            report_id: REPORT_ID_MULTITOUCH,
            scan_time: 0,
            contact_count: 0,
            is_button_clicked: false,
            contacts: [PtpContact::default(); PTP_MAX_CONTACT_POINTS],
        }
    }
}

pub fn parse_report(input: &[u8], config: ParserConfig) -> Result<PtpReport, ParseError> {
    let Some((&report_id, rest)) = input.split_first() else {
        return Err(ParseError::Empty);
    };

    match report_id {
        REPORT_ID_MOUSE if input.len() > MOUSE_REPORT_SIZE => {
            parse_mt2_multitouch(&input[MOUSE_REPORT_SIZE..], config)
        }
        REPORT_ID_MOUSE => Err(ParseError::UnsupportedReport(REPORT_ID_MOUSE)),
        REPORT_ID_MULTITOUCH => parse_mt2_multitouch(input, config),
        _ => Err(ParseError::UnsupportedReport(rest.first().copied().unwrap_or(report_id))),
    }
}

pub fn parse_mt2_multitouch(input: &[u8], config: ParserConfig) -> Result<PtpReport, ParseError> {
    if input.len() < MT2_REPORT_HEADER_SIZE {
        return Err(ParseError::MalformedLength);
    }

    if input[0] != REPORT_ID_MULTITOUCH {
        return Err(ParseError::UnsupportedReport(input[0]));
    }

    let finger_bytes = &input[MT2_REPORT_HEADER_SIZE..];
    if finger_bytes.len() % MT2_FINGER_SIZE != 0 {
        return Err(ParseError::MalformedLength);
    }

    let flags = input[1];
    let timestamp_low = (flags >> 3) as u16;
    let timestamp_high = u16::from_le_bytes([input[2], input[3]]);
    let timestamp = (timestamp_high << 5) | timestamp_low;

    let mut report = PtpReport {
        scan_time: timestamp.wrapping_mul(10),
        is_button_clicked: (flags & 0x01) != 0 && !config.settings.button_disabled,
        ..PtpReport::default()
    };

    let raw_count = finger_bytes.len() / MT2_FINGER_SIZE;
    let count = raw_count.min(PTP_MAX_CONTACT_POINTS);
    report.contact_count = count as u8;

    for index in 0..count {
        let start = index * MT2_FINGER_SIZE;
        let finger = parse_finger(&finger_bytes[start..start + MT2_FINGER_SIZE]);
        report.contacts[index] = finger_to_contact(finger, config);
    }

    Ok(report)
}

pub fn parse_finger(input: &[u8]) -> Mt2Finger {
    debug_assert!(input.len() >= MT2_FINGER_SIZE);

    let packed = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
    let x = sign_extend_13((packed & 0x1fff) as u16);
    let y = sign_extend_13(((packed >> 13) & 0x1fff) as u16);
    let packed_tail = input[8];

    Mt2Finger {
        absolute_x: x,
        absolute_y: y,
        finger: ((packed >> 26) & 0x07) as u8,
        state: ((packed >> 29) & 0x07) as u8,
        touch_major: input[4],
        touch_minor: input[5],
        size: input[6],
        pressure: input[7],
        id: packed_tail & 0x0f,
        orientation: packed_tail >> 5,
    }
}

pub fn finger_to_contact(finger: Mt2Finger, config: ParserConfig) -> PtpContact {
    let x = saturating_axis(finger.absolute_x, config.x.min);
    let y = saturating_axis(-finger.absolute_y, config.y.min);
    let is_near = (finger.state & 0x02) != 0;
    let is_valid = (finger.state & 0x04) != 0;

    PtpContact {
        confidence: !config.settings.palm_rejection || finger.finger != 6,
        tip_switch: is_valid && (!config.settings.ignore_near_fingers || !is_near),
        contact_id: finger.id as u32,
        x,
        y,
    }
}

fn sign_extend_13(value: u16) -> i16 {
    ((value << 3) as i16) >> 3
}

fn saturating_axis(value: i16, min: i16) -> u16 {
    value.saturating_sub(min).max(0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_type5_finger_bitfields() {
        let packed = 1234_u32 | (567_u32 << 13) | (6_u32 << 26) | (4_u32 << 29);
        let mut bytes = [0_u8; MT2_FINGER_SIZE];
        bytes[0..4].copy_from_slice(&packed.to_le_bytes());
        bytes[4] = 10;
        bytes[5] = 11;
        bytes[6] = 12;
        bytes[7] = 13;
        bytes[8] = 7 | (3 << 5);

        let finger = parse_finger(&bytes);

        assert_eq!(finger.absolute_x, 1234);
        assert_eq!(finger.absolute_y, 567);
        assert_eq!(finger.finger, 6);
        assert_eq!(finger.state, 4);
        assert_eq!(finger.id, 7);
        assert_eq!(finger.orientation, 3);
    }

    #[test]
    fn parses_multitouch_report() {
        let packed = 100_u32 | (50_u32 << 13) | (1_u32 << 26) | (4_u32 << 29);
        let mut report = [0_u8; MT2_REPORT_HEADER_SIZE + MT2_FINGER_SIZE];
        report[0] = REPORT_ID_MULTITOUCH;
        report[1] = 0x01 | (3 << 3);
        report[2..4].copy_from_slice(&2_u16.to_le_bytes());
        report[4..8].copy_from_slice(&packed.to_le_bytes());
        report[12] = 9;

        let parsed = parse_report(&report, ParserConfig::default()).unwrap();

        assert!(parsed.is_button_clicked);
        assert_eq!(parsed.scan_time, ((2 << 5) | 3) * 10);
        assert_eq!(parsed.contact_count, 1);
        assert_eq!(parsed.contacts[0].contact_id, 9);
        assert_eq!(parsed.contacts[0].x, 100);
        assert_eq!(parsed.contacts[0].tip_switch, true);
    }
}
