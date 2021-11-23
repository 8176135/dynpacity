use windows::core;
use windows::Win32::UI::Accessibility;
use windows::Win32::UI::WindowsAndMessaging;
use windows::Win32::Foundation::{HWND, BOOL, LPARAM};

use std::{
    ops::{BitAnd, BitOr, BitXor, Not},
    time::Duration,
};
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, MSG};

#[derive(Debug, Copy, Clone)]
enum LoopAction {
	DimAllWindows,
	ResetAllWindows,
}
#[derive(Debug, Copy, Clone)]
struct LoopAllWindowParams {
	action: LoopAction,
	active_hwnd: HWND,
}

const DIMMING_BRIGHTNESS: f32 = 0.5;
static CURRENT_ACTIVE_WINDOW: AtomicIsize = AtomicIsize::new(0);

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

	ctrlc::set_handler(|| unsafe {
		let data = Box::new(LoopAllWindowParams {
			active_hwnd: WindowsAndMessaging::GetForegroundWindow(),
			action: LoopAction::ResetAllWindows,
		});
		let raw_data = Box::into_raw(data);
		let res = WindowsAndMessaging::EnumWindows(Some(loop_all_windows), LPARAM(raw_data as isize));
		Box::from_raw(raw_data); // Cleanup memory
		if res.as_bool() {
			std::process::exit(0);
		} else {
			eprintln!("Failed to loop through all windows on exit");
			std::process::exit(1);
		}
	}).expect("Failed to set ctrl c handler");

	unsafe {
		let active_window = WindowsAndMessaging::GetForegroundWindow();
		CURRENT_ACTIVE_WINDOW.store(active_window.0, Ordering::SeqCst);
		let data = Box::new(LoopAllWindowParams {
			active_hwnd: active_window,
			action: LoopAction::DimAllWindows
		});
		let raw_data = Box::into_raw(data);
		WindowsAndMessaging::EnumWindows(Some(loop_all_windows), LPARAM(raw_data as isize));
		Box::from_raw(raw_data); // Cleanup memory
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

unsafe extern "system" fn loop_all_windows(hwnd: HWND, param1: LPARAM) -> BOOL {
	let data = param1.0 as *mut LoopAllWindowParams;
	if WindowsAndMessaging::IsWindowVisible(hwnd).as_bool() {
		match (*data).action {
			LoopAction::DimAllWindows => {
				if (*data).active_hwnd != hwnd {
					change_brightness_window(hwnd, DIMMING_BRIGHTNESS)
				}
			},
			LoopAction::ResetAllWindows => reset_brightness_window(hwnd),
		}
	}
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
	let active_window = CURRENT_ACTIVE_WINDOW.swap(hwnd.0, Ordering::SeqCst);
	if active_window == hwnd.0 {
		return;
	}

	reset_brightness_window(hwnd);
	change_brightness_window(HWND(active_window), DIMMING_BRIGHTNESS);
}

unsafe fn change_brightness_window(hwnd: HWND, change_val: f32) {
    // let active_window = WindowsAndMessaging::GetForegroundWindow();
    let existing_layer =
        WindowsAndMessaging::GetWindowLongA(hwnd, WindowsAndMessaging::GWL_EXSTYLE);

    // let mut alpha_value = 0u8;
    if existing_layer.bitand(WindowsAndMessaging::WS_EX_LAYERED.0 as i32) == 0 {
        println!("Setting window to layered");
        WindowsAndMessaging::SetWindowLongA(
			hwnd,
            WindowsAndMessaging::GWL_EXSTYLE,
            existing_layer.bitor(WindowsAndMessaging::WS_EX_LAYERED.0 as i32),
        );
        // alpha_value = 255;
    }

    // if alpha_value != 255 {
    //     let res = WindowsAndMessaging::GetLayeredWindowAttributes(
    //         active_window,
    //         std::ptr::null_mut(),
    //         &mut alpha_value,
    //         std::ptr::null_mut(),
    //     );
	//
    //     if !res.as_bool() {
    //         eprintln!("Unable to get transparency value");
    //         return;
    //     }
    // }

    // let max_transparency_value = (u8::MAX as f32).powf(2.2);

    // let alpha_in_decimal = (alpha_value as f32).powf(2.2) / max_transparency_value;
    // println!("Current alpha value: {}", alpha_in_decimal);
    // let value_to_set = (((alpha_in_decimal + change_val) * max_transparency_value).powf(1.0 / 2.2)).clamp(0.0, 255.0);
    let res = WindowsAndMessaging::SetLayeredWindowAttributes(
		hwnd,
        u32::MAX,
		(255.0 * change_val) as u8,
        WindowsAndMessaging::LWA_ALPHA,
    );

    if !res.as_bool() {
        eprintln!("Unable to set transparency value");
        return;
    }

    // println!("Successfully set transparency to {}", value_to_set as u8);
}

unsafe fn reset_brightness_window(hwnd: HWND) {
    // let active_window = WindowsAndMessaging::GetForegroundWindow();

    // let existing_layer = WindowsAndMessaging::GetWindowLongA(active_window, WindowsAndMessaging::GWL_EXSTYLE);
    let res = WindowsAndMessaging::SetLayeredWindowAttributes(
		hwnd,
        u32::MAX,
        (1.0 * u8::MAX as f32) as u8,
        WindowsAndMessaging::LWA_ALPHA,
    );
    // dbg!(res);
    // WindowsAndMessaging::SetWindowLongA(active_window, WindowsAndMessaging::GWL_EXSTYLE,  existing_layer.bitor(WindowsAndMessaging::WS_EX_LAYERED.0.not() as i32));
}
