//! This module naively converts strings to UTF-16 for windows FFI, even though
//! Windows does not follow to UTF-16 implementation very well. There might be
//! some issues with this approach.
//!
//! For more robust implementation it would be better to use [OsString,
//! OsStringExt and OsStrExt](https://doc.rust-lang.org/std/os/windows/ffi/index.html)

/// Returns wchar vector
///
/// Usage with winapi: wchar("Foo").as_ptr() gives LPCWSTR
pub fn wchar(string: &str) -> Vec<u16> {
    format!("{}\0", string).encode_utf16().collect::<Vec<_>>()
}

/// Copies string to WCHAR array, ensuring that array has null terminator
///
/// Use this if winapi struct of certain size requires WCHAR array
pub fn wchar_array(string: &str, dst: &mut [u16]) {
    let mut s = string.encode_utf16().collect::<Vec<_>>();

    // Truncate utf16 array to fit in the buffer with null terminator
    s.truncate(dst.len() - 1);

    dst[..s.len()].copy_from_slice(s.as_slice());

    // Null terminator
    dst[s.len()] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that too long strings gets truncated and is null terminated
    #[test]
    fn test_wchar_too_long() {
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("HELLO WORLD", dst.as_mut());
        assert_eq!(dst, [72, 69, 76, 76, 0]);
    }

    /// Ensure that too short strings is null terminated
    #[test]
    fn test_wchar_too_short() {
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("HI!", dst.as_mut());
        assert_eq!(dst, [72, 73, 33, 0, 99]);
    }

    /// Ensure that empty string is null terminated
    #[test]
    fn test_wchar_empty() {
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("", dst.as_mut());
        assert_eq!(dst, [0, 99, 99, 99, 99]);
    }
}
