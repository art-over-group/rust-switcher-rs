#![allow(dead_code)]

//! Low-level clipboard primitives for a future controlled selection fallback.

use std::ptr;
use windows::Win32::{
    Foundation::{HGLOBAL, HWND},
    System::{
        DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData},
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        Ole::CF_UNICODETEXT,
    },
};

pub struct ClipboardGuard;

impl ClipboardGuard {
    pub fn open(hwnd: usize) -> windows::core::Result<Self> {
        unsafe { OpenClipboard(Some(HWND(hwnd as *mut _))) }?;
        Ok(Self)
    }
    pub fn open_foreground() -> windows::core::Result<Self> {
        unsafe { OpenClipboard(None) }?;
        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

pub fn read_text() -> windows::core::Result<Option<String>> {
    let handle = unsafe { GetClipboardData(CF_UNICODETEXT.0 as u32) }?;
    let handle = HGLOBAL(handle.0);
    if handle.0.is_null() {
        return Ok(None);
    }
    unsafe {
        let ptr = GlobalLock(handle) as *const u16;
        if ptr.is_null() {
            return Ok(None);
        }
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let units = ptr::slice_from_raw_parts(ptr, len);
        let text = String::from_utf16_lossy(&*units);
        let _ = GlobalUnlock(handle);
        Ok(Some(text))
    }
}

pub fn write_text(text: &str) -> windows::core::Result<()> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes = units.len() * std::mem::size_of::<u16>();
    let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) }?;
    unsafe {
        let raw = GlobalLock(handle) as *mut u16;
        if raw.is_null() {
            let _ = GlobalUnlock(handle);
            return Err(windows::core::Error::from_thread());
        }
        ptr::copy_nonoverlapping(units.as_ptr(), raw, units.len());
        let _ = GlobalUnlock(handle);
    }
    // Transfer handle ownership to clipboard.
    unsafe {
        let _guard = ClipboardGuard::open_foreground()?;
        EmptyClipboard()?;
        let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(windows::Win32::Foundation::HANDLE(handle.0)))?;
    }
    Ok(())
}

pub fn clear() -> windows::core::Result<()> {
    let _guard = ClipboardGuard::open_foreground()?;
    unsafe { EmptyClipboard() }?;
    Ok(())
}