//! Per-app exception rules for AutoConvert.
//!
//! Resolves the foreground application's executable file name and checks it
//! against the configured exclusion list. Used to keep AutoConvert away from
//! terminals and other apps where an unexpected `SendInput` correction would
//! be harmful (see `docs/features/per-app-exceptions.md`).
//!
//! The resolution deliberately has no cache: it runs only when AutoConvert
//! actually attempts a correction (a word completed by a delimiter), not on
//! every keystroke, so the two syscalls cost nothing — and skipping the cache
//! avoids stale `pid → exe` entries after PID reuse.

use windows::Win32::{
    Foundation::HWND,
    System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

/// Pure matcher: does the foreground executable match one rule?
///
/// Matching is case-insensitive and normalizes the `.exe` suffix on both
/// sides, so `cmd`, `cmd.exe` and `CMD.EXE` are equivalent. Rules are exact
/// file names — substring matching is deliberately avoided to prevent
/// surprises like `notepad.exe` matching `unotepad.exe`.
pub fn exe_matches_rule(exe_file_name: &str, rule: &str) -> bool {
    let rule = rule.trim().to_lowercase();
    if rule.is_empty() {
        return false;
    }
    let exe = exe_file_name.trim().to_lowercase();
    if exe.is_empty() {
        return false;
    }

    let exe_stem = exe.strip_suffix(".exe").unwrap_or(&exe);
    let rule_stem = rule.strip_suffix(".exe").unwrap_or(&rule);
    exe_stem == rule_stem
}

/// Whether the foreground app is excluded by the `excluded_apps` list.
pub fn foreground_app_excluded() -> bool {
    let Some(exe) = foreground_exe_file_name() else {
        return false;
    };

    let rules = crate::config::load().unwrap_or_default().excluded_apps;
    rules.iter().any(|rule| exe_matches_rule(&exe, rule))
}

/// Resolves the executable file name of the foreground window's process.
fn foreground_exe_file_name() -> Option<String> {
    unsafe {
        let foreground: HWND = GetForegroundWindow();
        if foreground.0.is_null() {
            return None;
        }

        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(foreground, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 1_024];
        let mut len = u32::try_from(buffer.len()).unwrap_or(1_024);
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &raw mut len,
        );
        if result.is_err() {
            return None;
        }

        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        let file_name = path.rsplit(['\\', '/']).next()?;
        (!file_name.is_empty()).then(|| file_name.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_case_insensitively() {
        assert!(exe_matches_rule("cmd.exe", "CMD.EXE"));
        assert!(exe_matches_rule("CMD.EXE", "cmd"));
        assert!(exe_matches_rule("WindowsTerminal.exe", "windowsterminal"));
    }

    #[test]
    fn requires_exact_name() {
        assert!(exe_matches_rule("cmd.exe", "cmd"));
        assert!(!exe_matches_rule("unotepad.exe", "notepad"));
        assert!(!exe_matches_rule("notepad++.exe", "notepad"));
    }

    #[test]
    fn ignores_empty_or_whitespace_rules() {
        assert!(!exe_matches_rule("cmd.exe", ""));
        assert!(!exe_matches_rule("cmd.exe", "   "));
        assert!(!exe_matches_rule("", "cmd"));
    }

    #[test]
    fn keeps_directory_suffixes_strict() {
        // A rule naming a directory-ish string must not match a file name.
        assert!(!exe_matches_rule("cmd.exe", "system32/cmd"));
    }
}
