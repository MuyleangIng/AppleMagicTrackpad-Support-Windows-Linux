#[cfg(target_os = "windows")]
mod windows_app {
    use std::ffi::{c_void, OsStr};
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use std::process::Command;
    use std::ptr;

    use mt2_settings::{preset_settings, save_settings, Settings};

    type Bool = i32;
    type Dword = u32;
    type Hbrush = *mut c_void;
    type Hcursor = *mut c_void;
    type Hicon = *mut c_void;
    type Hinstance = *mut c_void;
    type Hmenu = *mut c_void;
    type Hwnd = *mut c_void;
    type Lparam = isize;
    type Lresult = isize;
    type Uint = u32;
    type Wparam = usize;

    const CW_USEDEFAULT: i32 = 0x80000000_u32 as i32;
    const WS_OVERLAPPEDWINDOW: Dword = 0x00cf0000;
    const WS_VISIBLE: Dword = 0x10000000;
    const WS_CHILD: Dword = 0x40000000;
    const WS_TABSTOP: Dword = 0x00010000;
    const WS_GROUP: Dword = 0x00020000;
    const BS_PUSHBUTTON: Dword = 0x00000000;
    const SS_LEFT: Dword = 0x00000000;
    const WM_DESTROY: Uint = 0x0002;
    const WM_COMMAND: Uint = 0x0111;
    const SW_SHOW: i32 = 5;
    const COLOR_WINDOW: isize = 5;
    const IDC_ARROW: *const u16 = 32512 as *const u16;

    const ID_DEFAULTS: usize = 1001;
    const ID_LIGHT: usize = 1002;
    const ID_MEDIUM: usize = 1003;
    const ID_FIRM: usize = 1004;
    const ID_SILENT: usize = 1005;
    const ID_DISABLED: usize = 1006;
    const ID_MAXIMUM: usize = 1007;
    const ID_TOUCHPAD_SETTINGS: usize = 1008;

    #[repr(C)]
    struct WndClassW {
        style: Uint,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: Hinstance,
        h_icon: Hicon,
        h_cursor: Hcursor,
        hbr_background: Hbrush,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
    }

    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        pt_x: i32,
        pt_y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(class: *const WndClassW) -> u16;
        fn CreateWindowExW(
            ex_style: Dword,
            class_name: *const u16,
            window_name: *const u16,
            style: Dword,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: Hmenu,
            instance: Hinstance,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn DispatchMessageW(msg: *const Msg) -> Lresult;
        fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min_filter: Uint, max_filter: Uint) -> Bool;
        fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Hcursor;
        fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, flags: Uint) -> i32;
        fn PostQuitMessage(exit_code: i32);
        fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> Bool;
        fn TranslateMessage(msg: *const Msg) -> Bool;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
    }

    pub fn run() -> Result<(), String> {
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class_name = wide("Mt2WinUiWindow");
            let title = wide("Magic Trackpad 2");
            let wnd_class = WndClassW {
                style: 0,
                lpfn_wnd_proc: Some(window_proc),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: instance,
                h_icon: ptr::null_mut(),
                h_cursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                hbr_background: (COLOR_WINDOW + 1) as Hbrush,
                lpsz_menu_name: ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
            };

            if RegisterClassW(&wnd_class) == 0 {
                return Err("RegisterClassW failed".to_string());
            }

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                620,
                480,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                return Err("CreateWindowExW failed".to_string());
            }

            build_controls(hwnd, instance);
            ShowWindow(hwnd, SW_SHOW);

            let mut msg = Msg {
                hwnd: ptr::null_mut(),
                message: 0,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt_x: 0,
                pt_y: 0,
            };

            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    unsafe extern "system" fn window_proc(
        hwnd: Hwnd,
        msg: Uint,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        match msg {
            WM_COMMAND => {
                let id = w_param & 0xffff;
                handle_command(hwnd, id);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }

    unsafe fn build_controls(parent: Hwnd, instance: Hinstance) {
        add_static(parent, instance, "Magic Trackpad 2", 24, 22, 540, 30);
        add_static(
            parent,
            instance,
            "Rust Windows desktop UI for driver settings presets.",
            26,
            56,
            540,
            24,
        );
        add_static(
            parent,
            instance,
            "Run as Administrator so Windows allows registry writes.",
            26,
            84,
            540,
            24,
        );

        add_button(parent, instance, "Defaults", ID_DEFAULTS, 26, 130, 150, 36);
        add_button(parent, instance, "macOS Light", ID_LIGHT, 192, 130, 150, 36);
        add_button(parent, instance, "macOS Medium", ID_MEDIUM, 358, 130, 150, 36);
        add_button(parent, instance, "macOS Firm", ID_FIRM, 26, 182, 150, 36);
        add_button(parent, instance, "Silent", ID_SILENT, 192, 182, 150, 36);
        add_button(parent, instance, "Disabled", ID_DISABLED, 358, 182, 150, 36);
        add_button(parent, instance, "Maximum", ID_MAXIMUM, 26, 234, 150, 36);
        add_button(
            parent,
            instance,
            "Windows Touchpad Settings",
            ID_TOUCHPAD_SETTINGS,
            192,
            234,
            240,
            36,
        );

        add_static(
            parent,
            instance,
            "After applying a preset, reconnect the USB trackpad. Bluetooth live reload will be added after the Rust IOCTL path is ported.",
            26,
            310,
            540,
            60,
        );
    }

    unsafe fn add_static(parent: Hwnd, instance: Hinstance, text: &str, x: i32, y: i32, w: i32, h: i32) {
        let class = wide("STATIC");
        let text = wide(text);
        CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            x,
            y,
            w,
            h,
            parent,
            ptr::null_mut(),
            instance,
            ptr::null_mut(),
        );
    }

    unsafe fn add_button(
        parent: Hwnd,
        instance: Hinstance,
        text: &str,
        id: usize,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        let class = wide("BUTTON");
        let text = wide(text);
        CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_PUSHBUTTON,
            x,
            y,
            w,
            h,
            parent,
            id as Hmenu,
            instance,
            ptr::null_mut(),
        );
    }

    unsafe fn handle_command(hwnd: Hwnd, id: usize) {
        let result = match id {
            ID_DEFAULTS => save_settings(Settings::default()),
            ID_LIGHT => preset_settings("macos-light").and_then(save_settings),
            ID_MEDIUM => preset_settings("macos-medium").and_then(save_settings),
            ID_FIRM => preset_settings("macos-firm").and_then(save_settings),
            ID_SILENT => preset_settings("silent").and_then(save_settings),
            ID_DISABLED => preset_settings("disabled").and_then(save_settings),
            ID_MAXIMUM => preset_settings("maximum").and_then(save_settings),
            ID_TOUCHPAD_SETTINGS => {
                let _ = Command::new("cmd")
                    .args(["/C", "start", "", "ms-settings:devices-touchpad"])
                    .status();
                Ok(())
            }
            _ => Ok(()),
        };

        match result {
            Ok(()) if id != ID_TOUCHPAD_SETTINGS => message(hwnd, "Settings saved."),
            Ok(()) => {}
            Err(err) => message(hwnd, &err),
        }
    }

    unsafe fn message(hwnd: Hwnd, text: &str) {
        let text = wide(text);
        let title = wide("Magic Trackpad 2");
        MessageBoxW(hwnd, text.as_ptr(), title.as_ptr(), 0);
    }

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(iter::once(0))
            .collect()
    }
}

#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    match windows_app::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("mt2-win-ui is a Windows desktop app. Build and run it on Windows.");
}
