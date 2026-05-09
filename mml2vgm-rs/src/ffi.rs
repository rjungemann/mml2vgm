//! C FFI bindings for mml2vgm-rs
//!
//! This module provides a C-compatible interface for integrating with C/C++ code.
//! It implements the interface expected by tigerflame-vst3's chip_bindings.h
//!
//! Note: Function names use the `mml2vgm_` prefix to avoid conflicts with C++ code.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::chips::{ChipType, ChipInstance as RustChipInstance, create_chip};

// Opaque handle for chip instances (matches ChipHandle in C)
// This is a pointer to a Rust-allocated ChipInstance
#[repr(C)]
pub struct ChipHandle {
    // We use a raw pointer to avoid lifetime issues with FFI
    ptr: *mut RustChipInstance,
}

impl ChipHandle {
    fn new(instance: *mut RustChipInstance) -> Self {
        Self { ptr: instance }
    }
    
    fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

// Global initialization state
static INIT_STATE: OnceLock<bool> = OnceLock::new();

/// Initialize the chip binding layer
/// Returns 0 on success, -1 on failure
#[no_mangle]
pub extern "C" fn mml2vgm_chip_bindings_init() -> i32 {
    let _ = INIT_STATE.set(true);
    0
}

/// Shutdown the chip binding layer
#[no_mangle]
pub extern "C" fn mml2vgm_chip_bindings_shutdown() {
    // For now, just reset the init state
    // In a real implementation, we'd clean up any global resources
}

/// Create a new chip instance
/// Returns NULL on failure
#[no_mangle]
pub extern "C" fn mml2vgm_chip_create(type_ffi: i32, sample_rate: f32) -> *mut ChipHandle {
    let chip_type = match type_ffi.try_into() {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let instance = create_chip(chip_type, sample_rate as f64);
    if instance.is_none() {
        return std::ptr::null_mut();
    }
    
    let instance = Box::new(RustChipInstance::new(instance.unwrap(), sample_rate as f64));
    let handle = Box::new(ChipHandle::new(Box::into_raw(instance)));
    Box::into_raw(handle)
}

/// Destroy a chip instance
#[no_mangle]
pub extern "C" fn mml2vgm_chip_destroy(handle: *mut ChipHandle) {
    if handle.is_null() {
        return;
    }
    
    unsafe {
        let handle = Box::from_raw(handle);
        if !handle.is_null() {
            let _ = Box::from_raw(handle.ptr);
        }
    }
}

/// Process a single sample
/// left and right are pointers to float values to accumulate into
#[no_mangle]
pub extern "C" fn mml2vgm_chip_process_sample(
    handle: *mut ChipHandle,
    left: *mut f32,
    right: *mut f32,
) {
    if handle.is_null() || left.is_null() || right.is_null() {
        return;
    }
    
    unsafe {
        let chip_instance = &mut *(*handle).ptr;
        let (l, r) = chip_instance.render_sample();
        *left = l as f32;
        *right = r as f32;
    }
}

/// Reset a chip to initial state
#[no_mangle]
pub extern "C" fn mml2vgm_chip_reset(handle: *mut ChipHandle) {
    if handle.is_null() {
        return;
    }
    
    unsafe {
        let chip_instance = &mut *(*handle).ptr;
        chip_instance.reset();
    }
}

/// Get the number of parameters for a chip type
#[no_mangle]
pub extern "C" fn mml2vgm_chip_get_param_count(type_ffi: i32) -> i32 {
    let chip_type: ChipType = match type_ffi.try_into() {
        Ok(t) => t,
        Err(_) => return 0,
    };
    
    chip_type.param_count() as i32
}

/// Get parameter name
#[no_mangle]
pub extern "C" fn mml2vgm_chip_get_param_name(
    type_ffi: i32,
    param_id: i32,
) -> *const c_char {
    let chip_type: ChipType = match type_ffi.try_into() {
        Ok(t) => t,
        Err(_) => return std::ptr::null(),
    };
    
    let name = chip_type.param_name(param_id as usize);
    
    match CString::new(name) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null(),
    }
}

/// Get parameter value
#[no_mangle]
pub extern "C" fn mml2vgm_chip_get_param(
    handle: *mut ChipHandle,
    param_id: i32,
) -> f32 {
    if handle.is_null() {
        return 0.0;
    }
    
    unsafe {
        let chip_instance = &mut *(*handle).ptr;
        chip_instance.get_param(param_id as usize)
    }
}

/// Set parameter value
#[no_mangle]
pub extern "C" fn mml2vgm_chip_set_param(
    handle: *mut ChipHandle,
    param_id: i32,
    value: f32,
) {
    if handle.is_null() {
        return;
    }
    
    unsafe {
        let chip_instance = &mut *(*handle).ptr;
        chip_instance.set_param(param_id as usize, value);
    }
}

/// Get chip name
#[no_mangle]
pub extern "C" fn mml2vgm_chip_get_name(type_ffi: i32) -> *const c_char {
    let chip_type: ChipType = match type_ffi.try_into() {
        Ok(t) => t,
        Err(_) => return std::ptr::null(),
    };
    
    let name = chip_type.name();
    
    match CString::new(name) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null(),
    }
}

/// Get chip short name
#[no_mangle]
pub extern "C" fn mml2vgm_chip_get_short_name(type_ffi: i32) -> *const c_char {
    let chip_type: ChipType = match type_ffi.try_into() {
        Ok(t) => t,
        Err(_) => return std::ptr::null(),
    };
    
    let name = chip_type.short_name();
    
    match CString::new(name) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null(),
    }
}

// Helper to free C strings allocated by Rust
#[no_mangle]
pub extern "C" fn mml2vgm_chip_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
