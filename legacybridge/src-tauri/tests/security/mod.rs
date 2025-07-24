// Security test suite module
//
// Comprehensive security testing for LegacyBridge RTF conversion system

pub mod fuzzing_tests;
pub mod dos_resistance_tests;
pub mod injection_tests;
pub mod performance_security_tests;

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_security_suite_loads() {
        // Ensure all security test modules compile and load correctly
        println!("Security test suite loaded successfully");
    }
}