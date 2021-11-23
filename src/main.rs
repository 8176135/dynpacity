use windows::core;
use windows::Win32::UI::Accessibility;
use windows::Win32::UI::WindowsAndMessaging;
use windows::Win32::Foundation::{HWND, BOOL, LPARAM};

use std::{
    ops::{BitAnd, BitOr, BitXor, Not},
    time::Duration,
};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, MSG};

// use inputbot::KeybdKey as KK;

fn main() {
    // std::thread::sleep(Duration::from_millis(1000));

    // KK::Numrow1Key.bind(|| {
    //     // dbg!("EWOOWW");
    //     if KK::LControlKey.is_pressed() && KK::LShiftKey.is_pressed() {
    //         unsafe { change_brightness_window(-0.05) };
    //     }
    // });
    //
    // KK::Numrow2Key.bind(|| {
    //     // dbg!("EWOOWW");
    //     if KK::LControlKey.is_pressed() && KK::LShiftKey.is_pressed() {
    //         unsafe { change_brightness_window(0.05) };
    //     }
    // });
    //
    // KK::Numrow3Key.bind(|| {
    //     // dbg!("EWOOWW");
    //     if KK::LControlKey.is_pressed() && KK::LShiftKey.is_pressed() {
    //         unsafe { reset_brightness_window() };
    //     }
    // });
    //
    //
    //
    // inputbot::handle_input_events();

	unsafe {

		WindowsAndMessaging::EnumWindows(Some(loop_all_windows), 0);

		Accessibility::SetWinEventHook(
			WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND,
			WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND,
			None,
			Some(active_window_change),
			0,
			0,
			WindowsAndMessaging::WINEVENT_OUTOFCONTEXT | WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS,
		);

		let mut message = MSG::default();

		while GetMessageA(&mut message, HWND(0), 0, 0).into() {
			DispatchMessageA(&mut message);
		}
	}
}

unsafe extern "system" fn loop_all_windows(param0: HWND, param1: LPARAM) -> BOOL {

	return true.into()
}

unsafe extern "system" fn active_window_change(
    hwineventhook: Accessibility::HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    idchild: i32,
    ideventthread: u32,
    dwmseventtime: u32,
) {
	println!("Event: {}, HWND: {:x}", event, hwnd.0);
}

unsafe fn change_brightness_window(change_val: f32) {
    let active_window = WindowsAndMessaging::GetForegroundWindow();
    let existing_layer =
        WindowsAndMessaging::GetWindowLongA(active_window, WindowsAndMessaging::GWL_EXSTYLE);

    let mut alpha_value = 0u8;
    if existing_layer.bitand(WindowsAndMessaging::WS_EX_LAYERED.0 as i32) == 0 {
        println!("Setting window to layered");
        WindowsAndMessaging::SetWindowLongA(
            active_window,
            WindowsAndMessaging::GWL_EXSTYLE,
            existing_layer.bitor(WindowsAndMessaging::WS_EX_LAYERED.0 as i32),
        );
        alpha_value = 255;
    }

    if alpha_value != 255 {
        let res = WindowsAndMessaging::GetLayeredWindowAttributes(
            active_window,
            std::ptr::null_mut(),
            &mut alpha_value,
            std::ptr::null_mut(),
        );

        if !res.as_bool() {
            eprintln!("Unable to get transparency value");
            return;
        }
    }

    let max_transparency_value = (u8::MAX as f32).powf(2.2);

    // let alpha_in_decimal = (alpha_value as f32).powf(2.2) / max_transparency_value;
    // println!("Current alpha value: {}", alpha_in_decimal);
    // let value_to_set = (((alpha_in_decimal + change_val) * max_transparency_value).powf(1.0 / 2.2)).clamp(0.0, 255.0);
    let res = WindowsAndMessaging::SetLayeredWindowAttributes(
        active_window,
        u32::MAX,
        (alpha_value as i32 + if change_val > 0.0 { 10 } else { -10 }).clamp(0, 255) as u8,
        WindowsAndMessaging::LWA_ALPHA,
    );

    if !res.as_bool() {
        eprintln!("Unable to set transparency value");
        return;
    }

    // println!("Successfully set transparency to {}", value_to_set as u8);
}

unsafe fn reset_brightness_window() {
    let active_window = WindowsAndMessaging::GetForegroundWindow();

    // let existing_layer = WindowsAndMessaging::GetWindowLongA(active_window, WindowsAndMessaging::GWL_EXSTYLE);
    let res = WindowsAndMessaging::SetLayeredWindowAttributes(
        active_window,
        u32::MAX,
        (1.0 * u8::MAX as f32) as u8,
        WindowsAndMessaging::LWA_ALPHA,
    );
    dbg!(res);
    // WindowsAndMessaging::SetWindowLongA(active_window, WindowsAndMessaging::GWL_EXSTYLE,  existing_layer.bitor(WindowsAndMessaging::WS_EX_LAYERED.0.not() as i32));
}
