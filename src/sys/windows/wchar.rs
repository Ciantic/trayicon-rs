/// Returns wchar vector
///
/// Usage with winapi: wchar("Foo").as_ptr() gives LPCTSTR
pub fn wchar(string: &str) -> Vec<u16> {
    format!("{}\0", string).encode_utf16().collect::<Vec<_>>()
}

/// Copies string to WCHAR array, ensuring that array has null terminator
///
/// Use this if winapi struct of certain size requires WCHAR array
pub fn wchar_array(string: &str, dst: &mut [u16]) {
    let mut s = string.encode_utf16().collect::<Vec<_>>();
    let len = dst.len() - 1;
    s.truncate(len);
    dst[..s.len()].copy_from_slice(s.as_slice());
    dst[s.len()] = 0; // Null terminator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wchar_too_long() {
        // Ensure that too long strings gets truncated and is null terminated
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("HELLO WORLD", dst.as_mut());
        assert_eq!(dst, [72, 69, 76, 76, 0]);
    }

    #[test]
    fn test_wchar_too_short() {
        // Ensure that too long wchar gets truncated and is null terminated
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("HI!", dst.as_mut());
        assert_eq!(dst, [72, 73, 33, 0, 99]);
    }

    #[test]
    fn test_wchar_empty() {
        // Ensure that empty string is null terminated
        let mut dst: [u16; 5] = [99, 99, 99, 99, 99];
        wchar_array("", dst.as_mut());
        assert_eq!(dst, [0, 99, 99, 99, 99]);
    }
}
