//! Conversion feedback overlay.
//!
//! A frameless, click-through, topmost layered window that briefly shows the
//! converted text near the caret. Modeled on the notification queue pattern
//! (`platform/ui/notify.rs`): requests are pushed into a static queue and
//! drained in the main window procedure via [`WM_APP_OVERLAY`]. The overlay
//! window itself is created lazily on the UI thread and never takes focus.

use std::collections::VecDeque;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{
            Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute},
            Gdi::{
                BeginPaint, ClientToScreen, CreatePen, CreateSolidBrush, DT_CALCRECT,
                DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DrawTextW,
                EndPaint, GetDC, GetMonitorInfoW, HBRUSH, HFONT, HGDIOBJ, HMONITOR,
                MONITORINFO, MONITOR_DEFAULTTONEAREST, MonitorFromPoint, PAINTSTRUCT, PS_SOLID,
                ReleaseDC, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
            },
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetCursorPos, GetForegroundWindow, GetGUIThreadInfo,
            GetWindowTextW, GetWindowThreadProcessId, GUITHREADINFO, KillTimer, LWA_ALPHA,
            PostMessageW, RegisterClassExW, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            SetLayeredWindowAttributes, SetTimer, SetWindowPos, SetWindowTextW, ShowWindow,
            WM_APP, WM_NCDESTROY, WM_PAINT, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
    core::{PCWSTR, Result, w},
};

use crate::app::ConversionFeedback;
use crate::platform::ui::themes::Theme;
use crate::platform::win::state::with_state_mut;

pub const WM_APP_OVERLAY: u32 = WM_APP + 4;

const OVERLAY_CLASS: PCWSTR = w!("RustSwitcherOverlayWindow");
const OVERLAY_HIDE_TIMER_ID: usize = 0x5302;
const OVERLAY_VISIBLE_MS: u32 = 1_400;
const MAX_QUEUE_LEN: usize = 16;
const MAX_TEXT_CHARS: usize = 40;
const PAD_X: i32 = 16;
const PAD_Y: i32 = 10;
const GAP_ABOVE_CARET: i32 = 8;
const MAX_OVERLAY_W: i32 = 420;
const OVERLAY_ALPHA: u8 = 235;
const ROUND_CORNER: i32 = 12;

static QUEUE: OnceLock<Mutex<VecDeque<ConversionFeedback>>> = OnceLock::new();
static POSTED: AtomicBool = AtomicBool::new(false);
static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
static OVERLAY_HWND: Mutex<Option<isize>> = Mutex::new(None);
static DARK_THEME: AtomicU8 = AtomicU8::new(0);
static FONT: Mutex<Option<isize>> = Mutex::new(None);

/// Enqueues a conversion feedback request and posts the overlay message to
/// the main window. The presentation happens on the UI thread when the main
/// window procedure receives [`WM_APP_OVERLAY`].
pub fn request(hwnd: HWND, feedback: ConversionFeedback) {
    {
        let mut q = lock_queue();
        if q.len() >= MAX_QUEUE_LEN {
            q.pop_front();
        }
        q.push_back(feedback);
    }
    post_once(hwnd);
}

/// Drains the queue from the main window procedure and shows the overlay for
/// the latest request. Falls back to a tray balloon when the overlay cannot
/// be created (B4 toast fallback).
pub fn on_wm_app_overlay(main_hwnd: HWND) {
    POSTED.store(false, Ordering::Release);

    let mut latest = None;
    while let Some(feedback) = lock_queue().pop_front() {
        latest = Some(feedback);
    }
    let Some(feedback) = latest else {
        return;
    };

    let dark = with_state_mut(main_hwnd, |state| state.current_theme_dark).unwrap_or(false);
    DARK_THEME.store(u8::from(dark), Ordering::Release);

    tracing::trace!(source = ?feedback.source, "overlay: presenting conversion");

    if let Err(error) = show(main_hwnd, &feedback) {
        tracing::warn!(error = ?error, "overlay window unavailable, falling back to balloon");
        crate::platform::ui::notify::push_info(
            main_hwnd,
            "Rust Switcher",
            &display_text(&feedback),
        );
    }
}

fn queue() -> &'static Mutex<VecDeque<ConversionFeedback>> {
    QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn lock_queue() -> std::sync::MutexGuard<'static, VecDeque<ConversionFeedback>> {
    match queue().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn post_once(hwnd: HWND) {
    if POSTED.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe {
        if let Err(e) = PostMessageW(Some(hwnd), WM_APP_OVERLAY, WPARAM(0), LPARAM(0)) {
            tracing::warn!(error = ?e, "PostMessageW(WM_APP_OVERLAY) failed");
            POSTED.store(false, Ordering::Release);
        }
    }
}

/// One-line presentation text: `from → to`.
fn display_text(feedback: &ConversionFeedback) -> String {
    format!(
        "{} \u{2192} {}",
        truncate_text(&feedback.from),
        truncate_text(&feedback.to)
    )
}

/// Truncates feedback text to [`MAX_TEXT_CHARS`] characters with an ellipsis.
fn truncate_text(text: &str) -> String {
    if text.chars().count() <= MAX_TEXT_CHARS {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(MAX_TEXT_CHARS).collect();
    out.push('\u{2026}');
    out
}

fn overlay_hwnd() -> Option<HWND> {
    let slot = OVERLAY_HWND.lock().ok()?;
    let ptr = slot.filter(|ptr| *ptr != 0)?;
    Some(HWND(ptr as *mut core::ffi::c_void))
}

fn set_overlay_hwnd(hwnd: HWND) {
    if let Ok(mut slot) = OVERLAY_HWND.lock() {
        *slot = Some(hwnd.0 as isize);
    }
}

fn clear_overlay_hwnd() {
    if let Ok(mut slot) = OVERLAY_HWND.lock() {
        *slot = None;
    }
}

fn ensure_font() -> Option<HFONT> {
    if let Some(font) = FONT
        .lock()
        .ok()?
        .map(|ptr| HFONT(ptr as *mut core::ffi::c_void))
    {
        return Some(font);
    }
    let font = unsafe { super::visuals::create_message_font() }.ok()?;
    if let Ok(mut slot) = FONT.lock() {
        *slot = Some(font.0 as isize);
    }
    Some(font)
}

/// Creates the layered overlay window once, hidden, owned by `main_hwnd`.
fn ensure_overlay_window(main_hwnd: HWND) -> Result<HWND> {
    if let Some(existing) = overlay_hwnd() {
        return Ok(existing);
    }

    if !CLASS_REGISTERED.swap(true, Ordering::AcqRel) {
        let hinstance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(
            PCWSTR::null(),
        ) }?
        .into();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(overlay_wndproc),
            lpszClassName: OVERLAY_CLASS,
            hInstance: hinstance,
            ..Default::default()
        };
        unsafe {
            if RegisterClassExW(&raw const wc) == 0 {
                CLASS_REGISTERED.store(false, Ordering::Release);
                return Err(crate::utils::helpers::last_error());
            }
        }
    }

    let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW
        | WS_EX_TRANSPARENT;
    let hwnd = unsafe {
        CreateWindowExW(
            ex_style,
            OVERLAY_CLASS,
            w!(""),
            WS_POPUP,
            0,
            0,
            10,
            10,
            Some(main_hwnd),
            None,
            None,
            None,
        )
    }?;

    // Best effort Windows 11 rounded corners; older systems keep square edges.
    let preference = DWMWCP_ROUND;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&raw const preference).cast::<core::ffi::c_void>(),
            u32::try_from(std::mem::size_of_val(&preference)).unwrap_or(4),
        );
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), OVERLAY_ALPHA, LWA_ALPHA);
    }

    set_overlay_hwnd(hwnd);
    Ok(hwnd)
}

/// Sizes, positions, fills and schedules auto-hide for the overlay.
fn show(main_hwnd: HWND, feedback: &ConversionFeedback) -> Result<()> {
    let hwnd = ensure_overlay_window(main_hwnd)?;
    let text = display_text(feedback);

    unsafe {
        let _ = SetWindowTextW(hwnd, &windows::core::HSTRING::from(&text));

        let (width, height) = measure(hwnd, &text);
        let (x, y) = position_near_caret(width, height);

        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width,
            height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        let _ = SetTimer(Some(hwnd), OVERLAY_HIDE_TIMER_ID, OVERLAY_VISIBLE_MS, None);
    }
    Ok(())
}

/// Measures the composed text with the message font.
fn measure(hwnd: HWND, text: &str) -> (i32, i32) {
    let hdc = unsafe { GetDC(Some(hwnd)) };
    let size = measure_with_dc(hdc, text);
    unsafe { ReleaseDC(Some(hwnd), hdc) };
    size
}

fn measure_with_dc(hdc: windows::Win32::Graphics::Gdi::HDC, text: &str) -> (i32, i32) {
    let mut chars: Vec<u16> = text.encode_utf16().collect();
    let mut rect = RECT::default();

    unsafe {
        let old_font = ensure_font()
            .map(|font| SelectObject(hdc, HGDIOBJ::from(font)));
        DrawTextW(hdc, &mut chars, &raw mut rect, DT_CALCRECT | DT_NOPREFIX | DT_SINGLELINE);
        if let Some(old) = old_font {
            SelectObject(hdc, old);
        }
    }

    let text_w = rect.right - rect.left;
    let text_h = rect.bottom - rect.top;
    ((text_w + 2 * PAD_X).min(MAX_OVERLAY_W), text_h + 2 * PAD_Y)
}

/// Places the overlay above the caret (or cursor), clamped into the monitor
/// work area.
fn position_near_caret(width: i32, height: i32) -> (i32, i32) {
    let anchor = caret_screen_pos()
        .or_else(cursor_pos)
        .unwrap_or(POINT { x: 0, y: 0 });

    let mut x = anchor.x - width / 2;
    let mut y = anchor.y - height - GAP_ABOVE_CARET;

    let mut monitor = MONITORINFO {
        cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).unwrap_or(40),
        ..Default::default()
    };
    let hmonitor: HMONITOR = unsafe { MonitorFromPoint(anchor, MONITOR_DEFAULTTONEAREST) };
    let has_work_area =
        unsafe { GetMonitorInfoW(hmonitor, &raw mut monitor) }.as_bool();
    if has_work_area {
        let work = monitor.rcWork;
        x = x.clamp(work.left, (work.right - width).max(work.left));
        y = if y < work.top {
            anchor.y + GAP_ABOVE_CARET
        } else {
            y.clamp(work.top, (work.bottom - height).max(work.top))
        };
    }

    (x, y)
}

/// Screen position of the caret in the foreground window, when available.
fn caret_screen_pos() -> Option<POINT> {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return None;
        }
        let thread_id = GetWindowThreadProcessId(foreground, None);
        let mut info = GUITHREADINFO {
            cbSize: u32::try_from(std::mem::size_of::<GUITHREADINFO>()).unwrap_or(48),
            ..Default::default()
        };
        if GetGUIThreadInfo(thread_id, &raw mut info).is_err() || info.hwndCaret.0.is_null() {
            return None;
        }
        let mut point = POINT {
            x: info.rcCaret.left,
            y: info.rcCaret.bottom,
        };
        if ClientToScreen(info.hwndCaret, &raw mut point).as_bool() {
            Some(point)
        } else {
            None
        }
    }
}

fn cursor_pos() -> Option<POINT> {
    let mut point = POINT::default();
    let ok = unsafe { GetCursorPos(&raw mut point) }.is_ok();
    ok.then_some(point)
}

extern "system" fn overlay_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => paint_overlay(hwnd),
        WM_TIMER => {
            if wparam.0 == OVERLAY_HIDE_TIMER_ID {
                unsafe {
                    let _ = KillTimer(Some(hwnd), OVERLAY_HIDE_TIMER_ID);
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                return LRESULT(0);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_NCDESTROY => {
            clear_overlay_hwnd();
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn paint_overlay(hwnd: HWND) -> LRESULT {
    let dark = DARK_THEME.load(Ordering::Relaxed) != 0;
    let theme = if dark { Theme::Dark } else { Theme::Light };
    let colors = theme.colors();

    let mut text_buffer = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, &mut text_buffer) };
    let text = String::from_utf16_lossy(&text_buffer[..len.max(0) as usize]);

    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &raw mut ps);
        if hdc.is_invalid() {
            return LRESULT(0);
        }

        let mut client = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &raw mut client);

        let background: HBRUSH = CreateSolidBrush(colors.window_bg);
        let border = CreatePen(PS_SOLID, 1, theme.button_palette().border_normal);

        let old_brush = SelectObject(hdc, HGDIOBJ::from(background));
        let old_pen = SelectObject(hdc, HGDIOBJ::from(border));
        let _ = RoundRect(
            hdc,
            client.left,
            client.top,
            client.right,
            client.bottom,
            ROUND_CORNER,
            ROUND_CORNER,
        );
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);

        if let Some(font) = ensure_font() {
            SelectObject(hdc, HGDIOBJ::from(font));
        }
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text);

        let mut chars: Vec<u16> = text.encode_utf16().collect();
        let _ = DrawTextW(
            hdc,
            &mut chars,
            &raw mut client,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        let _ = windows::Win32::Graphics::Gdi::DeleteObject(HGDIOBJ::from(background));
        let _ = windows::Win32::Graphics::Gdi::DeleteObject(HGDIOBJ::from(border));
        let _ = EndPaint(hwnd, &raw mut ps);
    }

    LRESULT(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_short_text() {
        assert_eq!(truncate_text("ghbdtn"), "ghbdtn");
    }

    #[test]
    fn truncate_appends_ellipsis_for_long_text() {
        let long: String = "a".repeat(MAX_TEXT_CHARS + 10);
        let truncated = truncate_text(&long);
        assert_eq!(truncated.chars().count(), MAX_TEXT_CHARS + 1);
        assert!(truncated.ends_with('\u{2026}'));
    }

    #[test]
    fn truncate_handles_multibyte_characters() {
        let word = "\u{043F}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}";
        assert_eq!(truncate_text(word), word);
    }

    #[test]
    fn display_text_joins_from_and_to() {
        let feedback = ConversionFeedback::new(
            crate::app::ConversionSource::LastWord,
            "ghbdtn",
            "\u{043F}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}",
        );
        assert_eq!(
            display_text(&feedback),
            "ghbdtn \u{2192} \u{043F}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}"
        );
    }
}
