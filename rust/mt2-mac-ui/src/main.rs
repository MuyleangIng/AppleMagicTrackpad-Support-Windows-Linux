use std::ffi::{c_char, c_void, CString};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Err("mt2-mac-ui is for macOS only.".to_string());
    }

    let status = MacTrackpadStatus::load();
    unsafe { show_desktop_ui(&status) }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MacTrackpadStatus {
    detected: bool,
    name: String,
    transport: String,
    battery_percent: Option<u8>,
    serial_number: String,
    product_id: Option<u32>,
    firmware: String,
    tracking_speed: String,
    click_mode: String,
    tap_to_click: String,
    scroll_mode: String,
}

impl MacTrackpadStatus {
    fn load() -> Self {
        let clicking = read_trackpad_default("Clicking");
        let mut status = Self {
            name: "Magic Trackpad".to_string(),
            transport: "Unknown".to_string(),
            tracking_speed: read_tracking_speed(),
            click_mode: clicking
                .as_deref()
                .map(|value| {
                    if value == "1" {
                        "Tap/click enabled"
                    } else {
                        "Physical click"
                    }
                    .to_string()
                })
                .unwrap_or_else(|| "Unknown".to_string()),
            tap_to_click: clicking
                .as_deref()
                .map(|value| if value == "1" { "On" } else { "Off" }.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            scroll_mode: read_trackpad_default("TrackpadScroll")
                .map(|value| if value == "1" { "On" } else { "Off" }.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            ..Self::default()
        };

        if let Some(device) = read_trackpad_management_device() {
            status.detected = true;
            status.name = if device.product.trim().is_empty() {
                fallback_product_name(device.product_id)
            } else {
                device.product
            };
            status.transport = device.transport;
            status.battery_percent = device.battery_percent;
            status.serial_number = device.serial_number;
            status.product_id = Some(device.product_id);
            status.firmware = device.firmware;
        }

        if let Some(multitouch) = read_multitouch_device() {
            status.detected = true;
            if status.name == "Magic Trackpad" && !multitouch.product.trim().is_empty() {
                status.name = multitouch.product;
            }
            if status.transport == "Unknown" && !multitouch.transport.trim().is_empty() {
                status.transport = multitouch.transport;
            }
            if status.serial_number.is_empty() {
                status.serial_number = multitouch.serial_number;
            }
            if status.product_id.is_none() {
                status.product_id = Some(multitouch.product_id);
            }
            if status.tracking_speed == "Unknown" && !multitouch.pointer_multiplier.is_empty() {
                status.tracking_speed =
                    format!("Acceleration multiplier {}", multitouch.pointer_multiplier);
            }
        }

        status
    }

}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ManagementDevice {
    product: String,
    transport: String,
    battery_percent: Option<u8>,
    serial_number: String,
    product_id: u32,
    firmware: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MultitouchDevice {
    product: String,
    transport: String,
    serial_number: String,
    product_id: u32,
    pointer_multiplier: String,
}

type Id = *mut c_void;
type Sel = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
}

unsafe fn show_desktop_ui(status: &MacTrackpadStatus) -> Result<(), String> {
    let pool = msg_id(class("NSAutoreleasePool")?, "new")?;
    let app = msg_id(class("NSApplication")?, "sharedApplication")?;
    msg_void_i64(app, "setActivationPolicy:", 0)?;

    let window = create_window(status)?;
    msg_void_id(window, "makeKeyAndOrderFront:", std::ptr::null_mut())?;
    msg_void_bool(app, "activateIgnoringOtherApps:", true)?;
    msg_void(app, "run")?;

    msg_void(pool, "drain")?;
    Ok(())
}

unsafe fn create_window(status: &MacTrackpadStatus) -> Result<Id, String> {
    let style_titled = 1_u64;
    let style_closable = 1_u64 << 1;
    let style_miniaturizable = 1_u64 << 2;
    let style_resizable = 1_u64 << 3;
    let style = style_titled | style_closable | style_miniaturizable | style_resizable;
    let frame = rect(0.0, 0.0, 760.0, 620.0);

    let window = msg_id(class("NSWindow")?, "alloc")?;
    let window = msg_id_rect_u64_u64_bool(
        window,
        "initWithContentRect:styleMask:backing:defer:",
        frame,
        style,
        2,
        false,
    )?;
    msg_void_id(window, "setTitle:", ns_string("Magic Trackpad 2")?)?;
    msg_void_bool(window, "setReleasedWhenClosed:", false)?;
    msg_void(window, "center")?;

    let content = msg_id(window, "contentView")?;
    add_label(
        content,
        &status.name,
        rect(28.0, 560.0, 704.0, 34.0),
        28.0,
        true,
        false,
    )?;
    add_label(
        content,
        "Magic Trackpad status from macOS",
        rect(30.0, 535.0, 704.0, 22.0),
        14.0,
        false,
        true,
    )?;

    let status_text = if status.detected {
        "Connected"
    } else {
        "Not detected"
    };
    add_label(
        content,
        status_text,
        rect(30.0, 500.0, 180.0, 24.0),
        16.0,
        true,
        false,
    )?;

    let items = status_items(status);
    let start_y = 450.0;
    let row_h = 72.0;
    let col_w = 340.0;
    for (index, (label, value)) in items.iter().enumerate() {
        let col = index % 2;
        let row = index / 2;
        let x = 30.0 + (col as f64 * (col_w + 28.0));
        let y = start_y - (row as f64 * row_h);

        add_label(content, label, rect(x, y + 28.0, col_w, 18.0), 11.0, true, true)?;
        add_label(content, value, rect(x, y, col_w, 28.0), 18.0, false, false)?;
    }

    add_label(
        content,
        "Open macOS System Settings for detailed trackpad and Bluetooth controls.",
        rect(30.0, 26.0, 704.0, 22.0),
        12.0,
        false,
        true,
    )?;

    Ok(window)
}

unsafe fn add_label(
    parent: Id,
    text: &str,
    frame: CGRect,
    size: f64,
    bold: bool,
    secondary: bool,
) -> Result<Id, String> {
    let field = msg_id(class("NSTextField")?, "alloc")?;
    let field = msg_id_rect(field, "initWithFrame:", frame)?;
    msg_void_id(field, "setStringValue:", ns_string(text)?)?;
    msg_void_bool(field, "setBezeled:", false)?;
    msg_void_bool(field, "setDrawsBackground:", false)?;
    msg_void_bool(field, "setEditable:", false)?;
    msg_void_bool(field, "setSelectable:", true)?;

    let font_class = class("NSFont")?;
    let font = if bold {
        msg_id_f64(font_class, "boldSystemFontOfSize:", size)?
    } else {
        msg_id_f64(font_class, "systemFontOfSize:", size)?
    };
    msg_void_id(field, "setFont:", font)?;

    if secondary {
        let color = msg_id(class("NSColor")?, "secondaryLabelColor")?;
        msg_void_id(field, "setTextColor:", color)?;
    }

    msg_void_id(parent, "addSubview:", field)?;
    Ok(field)
}

fn status_items(status: &MacTrackpadStatus) -> Vec<(String, String)> {
    let connected = if status.detected { "Connected" } else { "Not detected" };
    let battery = status
        .battery_percent
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "Unknown".to_string());
    let product_id = status
        .product_id
        .map(|value| format!("0x{value:04x}"))
        .unwrap_or_else(|| "Unknown".to_string());
    let serial = value_or_unknown(&status.serial_number);
    let firmware = value_or_unknown(&status.firmware);

    vec![
        ("Status".to_string(), connected.to_string()),
        ("Transport".to_string(), status.transport.clone()),
        ("Battery".to_string(), battery),
        ("Speed mode".to_string(), status.tracking_speed.clone()),
        ("Tap to click".to_string(), status.tap_to_click.clone()),
        ("Click mode".to_string(), status.click_mode.clone()),
        ("Scrolling".to_string(), status.scroll_mode.clone()),
        ("Product ID".to_string(), product_id),
        ("Serial".to_string(), serial),
        ("Firmware".to_string(), firmware),
    ]
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> CGRect {
    CGRect {
        origin: CGPoint { x, y },
        size: CGSize { width, height },
    }
}

unsafe fn class(name: &str) -> Result<Id, String> {
    let name = CString::new(name).map_err(|err| err.to_string())?;
    let class = objc_getClass(name.as_ptr());
    if class.is_null() {
        Err("Objective-C class not found".to_string())
    } else {
        Ok(class)
    }
}

unsafe fn selector(name: &str) -> Result<Sel, String> {
    let name = CString::new(name).map_err(|err| err.to_string())?;
    Ok(sel_registerName(name.as_ptr()))
}

unsafe fn ns_string(value: &str) -> Result<Id, String> {
    let value = CString::new(value).map_err(|err| err.to_string())?;
    let string = msg_id(class("NSString")?, "alloc")?;
    msg_id_cstr(string, "initWithUTF8String:", value.as_ptr())
}

unsafe fn msg_id(receiver: Id, sel_name: &str) -> Result<Id, String> {
    let function: unsafe extern "C" fn(Id, Sel) -> Id = std::mem::transmute(objc_msgSend as *const ());
    Ok(function(receiver, selector(sel_name)?))
}

unsafe fn msg_id_cstr(receiver: Id, sel_name: &str, arg: *const c_char) -> Result<Id, String> {
    let function: unsafe extern "C" fn(Id, Sel, *const c_char) -> Id =
        std::mem::transmute(objc_msgSend as *const ());
    Ok(function(receiver, selector(sel_name)?, arg))
}

unsafe fn msg_id_rect(receiver: Id, sel_name: &str, frame: CGRect) -> Result<Id, String> {
    let function: unsafe extern "C" fn(Id, Sel, CGRect) -> Id =
        std::mem::transmute(objc_msgSend as *const ());
    Ok(function(receiver, selector(sel_name)?, frame))
}

unsafe fn msg_id_rect_u64_u64_bool(
    receiver: Id,
    sel_name: &str,
    frame: CGRect,
    style: u64,
    backing: u64,
    defer: bool,
) -> Result<Id, String> {
    let function: unsafe extern "C" fn(Id, Sel, CGRect, u64, u64, bool) -> Id =
        std::mem::transmute(objc_msgSend as *const ());
    Ok(function(receiver, selector(sel_name)?, frame, style, backing, defer))
}

unsafe fn msg_id_f64(receiver: Id, sel_name: &str, arg: f64) -> Result<Id, String> {
    let function: unsafe extern "C" fn(Id, Sel, f64) -> Id =
        std::mem::transmute(objc_msgSend as *const ());
    Ok(function(receiver, selector(sel_name)?, arg))
}

unsafe fn msg_void(receiver: Id, sel_name: &str) -> Result<(), String> {
    let function: unsafe extern "C" fn(Id, Sel) = std::mem::transmute(objc_msgSend as *const ());
    function(receiver, selector(sel_name)?);
    Ok(())
}

unsafe fn msg_void_id(receiver: Id, sel_name: &str, arg: Id) -> Result<(), String> {
    let function: unsafe extern "C" fn(Id, Sel, Id) =
        std::mem::transmute(objc_msgSend as *const ());
    function(receiver, selector(sel_name)?, arg);
    Ok(())
}

unsafe fn msg_void_bool(receiver: Id, sel_name: &str, arg: bool) -> Result<(), String> {
    let function: unsafe extern "C" fn(Id, Sel, bool) =
        std::mem::transmute(objc_msgSend as *const ());
    function(receiver, selector(sel_name)?, arg);
    Ok(())
}

unsafe fn msg_void_i64(receiver: Id, sel_name: &str, arg: i64) -> Result<(), String> {
    let function: unsafe extern "C" fn(Id, Sel, i64) =
        std::mem::transmute(objc_msgSend as *const ());
    function(receiver, selector(sel_name)?, arg);
    Ok(())
}

fn read_trackpad_management_device() -> Option<ManagementDevice> {
    let output = command_output("ioreg", &["-r", "-c", "AppleDeviceManagementHIDEventService"])?;
    split_ioreg_objects(&output)
        .into_iter()
        .find_map(|block| {
            let product_id = read_u32_property(block, "ProductID")?;
            if !is_magic_trackpad_product(product_id) {
                return None;
            }

            Some(ManagementDevice {
                product: read_string_property(block, "Product").unwrap_or_default(),
                transport: read_string_property(block, "Transport")
                    .unwrap_or_else(|| "Unknown".to_string()),
                battery_percent: read_u32_property(block, "BatteryPercent")
                    .and_then(|value| u8::try_from(value).ok()),
                serial_number: read_string_property(block, "SerialNumber").unwrap_or_default(),
                product_id,
                firmware: read_u32_property(block, "MTFW Version")
                    .or_else(|| read_u32_property(block, "BTFW Version"))
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            })
        })
}

fn read_multitouch_device() -> Option<MultitouchDevice> {
    let output = command_output("ioreg", &["-r", "-c", "AppleMultitouchDevice"])?;
    split_ioreg_objects(&output)
        .into_iter()
        .find_map(|block| {
            let product_id = read_u32_property(block, "ProductID")?;
            if !is_magic_trackpad_product(product_id) {
                return None;
            }

            Some(MultitouchDevice {
                product: read_string_property(block, "Product").unwrap_or_default(),
                transport: read_string_property(block, "Transport").unwrap_or_default(),
                serial_number: read_string_property(block, "SerialNumber").unwrap_or_default(),
                product_id,
                pointer_multiplier: read_u32_property(block, "HIDPointerAccelerationMultiplier")
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            })
        })
}

fn read_tracking_speed() -> String {
    command_output("defaults", &["read", "-g", "com.apple.trackpad.scaling"])
        .map(|value| format!("Tracking speed {}", value.trim()))
        .or_else(|| {
            command_output("defaults", &["read", "-g", "com.apple.mouse.scaling"])
                .map(|value| format!("Pointer speed {}", value.trim()))
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn read_trackpad_default(key: &str) -> Option<String> {
    command_output("defaults", &["read", "com.apple.AppleMultitouchTrackpad", key])
        .map(|value| value.trim().to_string())
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn split_ioreg_objects(output: &str) -> Vec<&str> {
    output
        .split("\n+-o ")
        .filter(|block| !block.trim().is_empty())
        .collect()
}

fn read_string_property(block: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\" = \"");
    let start = block.find(&needle)? + needle.len();
    let tail = &block[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn read_u32_property(block: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\" = ");
    let start = block.find(&needle)? + needle.len();
    let tail = &block[start..];
    let value = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    value.parse().ok()
}

fn is_magic_trackpad_product(product_id: u32) -> bool {
    matches!(product_id, 0x0265 | 0x0324)
}

fn fallback_product_name(product_id: u32) -> String {
    match product_id {
        0x0265 => "Magic Trackpad 2".to_string(),
        0x0324 => "Magic Trackpad USB-C".to_string(),
        _ => "Magic Trackpad".to_string(),
    }
}

fn value_or_unknown(value: &str) -> String {
    if value.trim().is_empty() {
        "Unknown".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ioreg_trackpad_properties() {
        let block = r#"
    {
      "Product" = ""
      "SerialNumber" = "30:82:16:F2:74:EA"
      "Transport" = "Bluetooth"
      "ProductID" = 613
      "BatteryPercent" = 66
      "MTFW Version" = 1301
    }
"#;

        assert_eq!(read_u32_property(block, "ProductID"), Some(613));
        assert_eq!(read_u32_property(block, "BatteryPercent"), Some(66));
        assert_eq!(
            read_string_property(block, "SerialNumber").as_deref(),
            Some("30:82:16:F2:74:EA")
        );
        assert!(is_magic_trackpad_product(613));
    }

    #[test]
    fn builds_status_items() {
        let status = MacTrackpadStatus {
            detected: true,
            name: "Magic Trackpad 2".to_string(),
            transport: "Bluetooth".to_string(),
            battery_percent: Some(66),
            tracking_speed: "Pointer speed 2".to_string(),
            ..MacTrackpadStatus::default()
        };

        let items = status_items(&status);

        assert!(items.contains(&("Status".to_string(), "Connected".to_string())));
        assert!(items.contains(&("Battery".to_string(), "66%".to_string())));
        assert!(items.contains(&("Speed mode".to_string(), "Pointer speed 2".to_string())));
    }
}
