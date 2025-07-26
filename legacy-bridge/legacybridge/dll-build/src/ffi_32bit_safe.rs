// 32-bit safe FFI enhancements for LegacyBridge
// Provides architecture-aware implementations and memory constraints

use std::os::raw::{c_char, c_int};
use std::ffi::CStr;
use std::sync::Mutex;

// Architecture-aware memory constraints
#[cfg(target_pointer_width = "32")]
const MAX_MEMORY_USAGE: usize = 500 * 1024 * 1024; // 500MB for 32-bit systems
#[cfg(target_pointer_width = "32")]
const MAX_STRING_SIZE: usize = 50 * 1024 * 1024;   // 50MB max string size
#[cfg(target_pointer_width = "32")]
const MAX_BATCH_SIZE: usize = 100;                  // Limit batch operations

#[cfg(target_pointer_width = "64")]
const MAX_MEMORY_USAGE: usize = 2 * 1024 * 1024 * 1024; // 2GB for 64-bit systems
#[cfg(target_pointer_width = "64")]
const MAX_STRING_SIZE: usize = 200 * 1024 * 1024;       // 200MB max string size
#[cfg(target_pointer_width = "64")]
const MAX_BATCH_SIZE: usize = 1000;                     // Higher batch limit

// 32-bit safe memory tracking
#[cfg(target_pointer_width = "32")]
static mut CURRENT_MEMORY_USAGE: usize = 0;

/// Check if allocation is safe for current architecture
#[inline]
pub fn is_allocation_safe(size: usize) -> bool {
    #[cfg(target_pointer_width = "32")]
    {
        unsafe {
            if CURRENT_MEMORY_USAGE + size > MAX_MEMORY_USAGE {
                return false;
            }
        }
    }
    size <= MAX_STRING_SIZE
}

/// Track memory allocation for 32-bit systems
#[cfg(target_pointer_width = "32")]
pub fn track_allocation(size: usize) {
    unsafe {
        CURRENT_MEMORY_USAGE += size;
    }
}

/// Track memory deallocation for 32-bit systems
#[cfg(target_pointer_width = "32")]
pub fn track_deallocation(size: usize) {
    unsafe {
        CURRENT_MEMORY_USAGE = CURRENT_MEMORY_USAGE.saturating_sub(size);
    }
}

/// 32-bit safe RTF to Markdown conversion
/// Uses u32 for lengths to ensure 32-bit compatibility
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_32bit_safe(
    rtf_content: *const c_char,
    output_length: *mut u32,  // Use u32 instead of usize for 32-bit compatibility
) -> *mut c_char {
    // Validate pointer
    if rtf_content.is_null() || output_length.is_null() {
        return std::ptr::null_mut();
    }
    
    // Get input length safely
    let input_len = match CStr::from_ptr(rtf_content).to_bytes().len() {
        len if len > MAX_STRING_SIZE => return std::ptr::null_mut(),
        len => len,
    };
    
    // Check if allocation is safe
    if !is_allocation_safe(input_len * 2) {  // Assume output might be 2x input
        return std::ptr::null_mut();
    }
    
    // Perform conversion with memory tracking
    #[cfg(target_pointer_width = "32")]
    track_allocation(input_len);
    
    // Call the actual conversion function
    let mut output_buffer: *mut c_char = std::ptr::null_mut();
    let mut output_len_c_int: c_int = 0;
    
    let result = crate::ffi::legacybridge_rtf_to_markdown(
        rtf_content,
        &mut output_buffer,
        &mut output_len_c_int,
    );
    
    if result == 0 && !output_buffer.is_null() {
        *output_length = output_len_c_int as u32;
        output_buffer
    } else {
        #[cfg(target_pointer_width = "32")]
        track_deallocation(input_len);
        std::ptr::null_mut()
    }
}

/// 32-bit safe batch operation with size limits
#[no_mangle]
pub unsafe extern "C" fn legacybridge_batch_rtf_to_markdown_32bit_safe(
    rtf_array: *const *const c_char,
    count: u32,  // Use u32 for 32-bit compatibility
    output_array: *mut *mut c_char,
    output_lengths: *mut u32,  // Use u32 array
) -> u32 {  // Return u32
    if rtf_array.is_null() || output_array.is_null() || output_lengths.is_null() {
        return 0;
    }
    
    // Enforce batch size limit based on architecture
    let safe_count = std::cmp::min(count as usize, MAX_BATCH_SIZE);
    
    let mut success_count = 0u32;
    
    for i in 0..safe_count {
        let rtf_ptr = *rtf_array.add(i);
        let output_ptr = output_array.add(i);
        let length_ptr = output_lengths.add(i);
        
        let result_ptr = legacybridge_rtf_to_markdown_32bit_safe(rtf_ptr, length_ptr);
        
        if !result_ptr.is_null() {
            *output_ptr = result_ptr;
            success_count += 1;
        } else {
            *output_ptr = std::ptr::null_mut();
            *length_ptr = 0;
        }
    }
    
    success_count
}

/// Get current memory usage (useful for monitoring on 32-bit systems)
#[no_mangle]
pub extern "C" fn legacybridge_get_memory_usage() -> u32 {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        CURRENT_MEMORY_USAGE as u32
    }
    
    #[cfg(target_pointer_width = "64")]
    0  // Not tracked on 64-bit systems
}

/// Get maximum safe string size for current architecture
#[no_mangle]
pub extern "C" fn legacybridge_get_max_string_size() -> u32 {
    MAX_STRING_SIZE as u32
}

/// Get architecture info (32 or 64)
#[no_mangle]
pub extern "C" fn legacybridge_get_architecture_bits() -> u32 {
    std::mem::size_of::<usize>() as u32 * 8
}

/// Arena allocator for legacy systems to reduce fragmentation
#[cfg(target_pointer_width = "32")]
pub struct LegacyArena {
    buffer: Vec<u8>,
    offset: usize,
}

#[cfg(target_pointer_width = "32")]
impl LegacyArena {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            offset: 0,
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        let aligned_size = (size + 7) & !7;  // 8-byte alignment
        
        if self.offset + aligned_size > self.buffer.capacity() {
            return None;
        }
        
        let ptr = unsafe {
            self.buffer.as_mut_ptr().add(self.offset)
        };
        self.offset += aligned_size;
        
        Some(ptr)
    }
    
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

// Static arena for 32-bit systems
#[cfg(target_pointer_width = "32")]
lazy_static::lazy_static! {
    static ref LEGACY_ARENA: Mutex<LegacyArena> = Mutex::new(LegacyArena::new(10 * 1024 * 1024)); // 10MB arena
}

/// Arena-based string allocation for 32-bit systems
#[cfg(target_pointer_width = "32")]
pub fn allocate_string_arena(s: &str) -> Option<*mut c_char> {
    if let Ok(mut arena) = LEGACY_ARENA.lock() {
        let bytes = s.as_bytes();
        let size = bytes.len() + 1;  // +1 for null terminator
        
        if let Some(ptr) = arena.allocate(size) {
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
                *ptr.add(bytes.len()) = 0;  // Null terminator
            }
            return Some(ptr as *mut c_char);
        }
    }
    None
}

/// Reset arena (should be called periodically on 32-bit systems)
#[cfg(target_pointer_width = "32")]
#[no_mangle]
pub extern "C" fn legacybridge_reset_memory_arena() {
    if let Ok(mut arena) = LEGACY_ARENA.lock() {
        arena.reset();
    }
}

// VB6-specific safe wrappers with explicit stdcall convention on Windows
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
pub mod vb6_safe {
    use super::*;
    
    /// VB6-safe wrapper with BSTR-like handling
    #[no_mangle]
    #[export_name = "legacybridge_rtf_to_markdown_vb6"]
    pub unsafe extern "stdcall" fn legacybridge_rtf_to_markdown_vb6(
        rtf_content: *const c_char,
    ) -> *mut c_char {
        let mut output_length: u32 = 0;
        legacybridge_rtf_to_markdown_32bit_safe(rtf_content, &mut output_length)
    }
    
    /// VB6-safe string length function
    #[no_mangle]
    #[export_name = "legacybridge_get_string_length_vb6"]
    pub unsafe extern "stdcall" fn legacybridge_get_string_length_vb6(
        str_ptr: *const c_char,
    ) -> u32 {
        if str_ptr.is_null() {
            return 0;
        }
        CStr::from_ptr(str_ptr).to_bytes().len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_architecture_detection() {
        let bits = legacybridge_get_architecture_bits();
        assert!(bits == 32 || bits == 64);
    }
    
    #[test]
    fn test_memory_constraints() {
        assert!(is_allocation_safe(1024));  // 1KB should always be safe
        assert!(!is_allocation_safe(usize::MAX));  // Max size should fail
    }
    
    #[test]
    #[cfg(target_pointer_width = "32")]
    fn test_32bit_memory_tracking() {
        unsafe {
            CURRENT_MEMORY_USAGE = 0;
        }
        
        track_allocation(1000);
        assert_eq!(legacybridge_get_memory_usage(), 1000);
        
        track_deallocation(500);
        assert_eq!(legacybridge_get_memory_usage(), 500);
    }
}