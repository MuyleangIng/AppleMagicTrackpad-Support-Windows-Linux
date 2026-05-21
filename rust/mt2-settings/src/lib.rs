#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub button_disabled: u32,
    pub feedback_click: u32,
    pub feedback_release: u32,
    pub stop_pressure: i32,
    pub stop_size: i32,
    pub ignore_button_finger: u32,
    pub ignore_near_fingers: u32,
    pub palm_rejection: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            button_disabled: 0,
            feedback_click: 0x060617,
            feedback_release: 0x000014,
            stop_pressure: 0,
            stop_size: -1,
            ignore_button_finger: 1,
            ignore_near_fingers: 1,
            palm_rejection: 1,
        }
    }
}

pub fn preset_settings(name: &str) -> Result<Settings, String> {
    let mut settings = Settings::default();

    match name {
        "macos-light" => {
            settings.feedback_click = 0x040415;
            settings.feedback_release = 0x000010;
        }
        "macos-medium" => {}
        "macos-firm" => {
            settings.feedback_click = 0x08081e;
            settings.feedback_release = 0x020218;
        }
        "silent" => {
            settings.feedback_click = 0x000017;
            settings.feedback_release = 0x000014;
        }
        "disabled" => {
            settings.button_disabled = 1;
            settings.feedback_click = 0;
            settings.feedback_release = 0;
        }
        "maximum" => {
            settings.feedback_click = 0x00ff_ffff;
            settings.feedback_release = 0x00ff_ffff;
        }
        _ => return Err(format!("unknown preset: {name}")),
    }

    Ok(settings)
}

pub fn save_settings(settings: Settings) -> Result<(), String> {
    platform::save_settings(settings)
}

#[cfg(target_os = "windows")]
mod platform {
    use super::Settings;
    use std::process::Command;

    pub fn save_settings(settings: Settings) -> Result<(), String> {
        let service_roots = [
            r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\WUDF\Services\AmtPtpDeviceUsbUm\Parameters",
            r"HKLM\SYSTEM\CurrentControlSet\Services\AmtPtpHidFilter\Parameters",
        ];

        for key in service_roots {
            reg_add(key, "ButtonDisabled", settings.button_disabled)?;
            reg_add(key, "FeedbackClick", settings.feedback_click)?;
            reg_add(key, "FeedbackRelease", settings.feedback_release)?;
            reg_add(key, "StopPressure", settings.stop_pressure)?;
            reg_add(key, "StopSize", settings.stop_size)?;
            reg_add(key, "IgnoreButtonFinger", settings.ignore_button_finger)?;
            reg_add(key, "IgnoreNearFingers", settings.ignore_near_fingers)?;
            reg_add(key, "PalmRejection", settings.palm_rejection)?;
        }

        Ok(())
    }

    fn reg_add<T: ToString>(key: &str, name: &str, value: T) -> Result<(), String> {
        let value = value.to_string();
        let status = Command::new("reg.exe")
            .args(["add", key, "/v", name, "/t", "REG_DWORD", "/d", &value, "/f"])
            .status()
            .map_err(|err| format!("failed to run reg.exe: {err}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("reg.exe failed while writing {key}\\{name}"))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::Settings;

    pub fn save_settings(_settings: Settings) -> Result<(), String> {
        Err("driver settings can be saved only on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_original_control_panel() {
        let settings = Settings::default();

        assert_eq!(settings.feedback_click, 0x060617);
        assert_eq!(settings.feedback_release, 0x000014);
        assert_eq!(settings.stop_pressure, 0);
        assert_eq!(settings.stop_size, -1);
    }

    #[test]
    fn maps_presets() {
        let disabled = preset_settings("disabled").unwrap();
        let firm = preset_settings("macos-firm").unwrap();

        assert_eq!(disabled.button_disabled, 1);
        assert_eq!(disabled.feedback_click, 0);
        assert_eq!(firm.feedback_click, 0x08081e);
        assert_eq!(firm.feedback_release, 0x020218);
    }
}
