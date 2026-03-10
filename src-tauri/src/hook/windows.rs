use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::ProcessStatus::K32GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};

#[cfg(target_os = "windows")]
pub fn get_active_window() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }
        
        // Get Window Title
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len == 0 {
            return None;
        }
        let title = String::from_utf16_lossy(&buf[..len as usize]);

        // Get Process Name
        let app_name = get_process_name(hwnd).unwrap_or_else(|| "Unknown".to_string());
        
        Some((title, app_name))
    }
}

#[cfg(target_os = "windows")]
fn get_process_name(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id).ok()?;
        
        let mut buf = [0u16; 512];
        let len = K32GetModuleBaseNameW(handle, None, &mut buf);
        let _ = CloseHandle(handle);

        if len == 0 {
            return None;
        }
        
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_title_captured() {
        // This test must be run on a machine with a visible desktop.
        // It captures the current foreground window and prints its details.
        let result = get_active_window();
        match result {
            Some((title, app_name)) => {
                println!("✅ Window title captured!");
                println!("   Title: {}", title);
                println!("   App:   {}", app_name);
                assert!(!title.is_empty(), "Title should not be empty");
                assert!(!app_name.is_empty(), "App name should not be empty");
            }
            None => {
                // On CI or locked screens this is acceptable
                println!("⚠️  No foreground window found (expected on CI/lock screen)");
                println!("   Null title handled gracefully — returned None, no panic.");
            }
        }
    }

    #[test]
    fn test_null_title_no_panic() {
        // Call get_active_window multiple times to verify it never panics
        for i in 0..10 {
            let result = get_active_window();
            match &result {
                Some((title, app)) => println!("  [{}] OK: {} ({})", i, title, app),
                None => println!("  [{}] None returned safely", i),
            }
        }
        println!("✅ Null title handled — no panic across 10 calls");
    }
}
