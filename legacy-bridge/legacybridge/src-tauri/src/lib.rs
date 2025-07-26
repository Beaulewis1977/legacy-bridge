// Library exports for legacybridge

pub mod conversion;
pub mod pipeline;
pub mod ffi;
pub mod ffi_error_bridge;
pub mod memory_pool_optimization;
pub mod panic_handler;

#[cfg(test)]
pub mod tests;

// Re-export main conversion functions
pub use conversion::{
    rtf_to_markdown,
    markdown_to_rtf,
    secure_rtf_to_markdown,
    secure_markdown_to_rtf,
};

// Re-export SIMD-optimized functions when available
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use conversion::simd_conversion::{
    rtf_to_markdown_simd,
    markdown_to_rtf_simd,
};

// Re-export error types
pub use conversion::{ConversionError, ConversionResult};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");