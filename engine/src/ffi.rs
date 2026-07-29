//! Raw Win32 kernel32 FFI declarations for USN Journal API.
//! Pure raw FFI, no external dependencies.

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// ===== Basic Types =====
pub type BOOL = i32;
pub type DWORD = u32;
pub type WORD = u16;
pub type HANDLE = isize;
pub type LPCWSTR = *const u16;
pub type LPVOID = *mut std::ffi::c_void;
pub type LPCVOID = *const std::ffi::c_void;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const INVALID_HANDLE_VALUE: isize = -1;

// ===== File Access Constants =====
pub const GENERIC_READ: DWORD = 0x80000000;
pub const GENERIC_WRITE: DWORD = 0x40000000;
pub const FILE_SHARE_READ: DWORD = 0x00000001;
pub const FILE_SHARE_WRITE: DWORD = 0x00000002;
pub const OPEN_EXISTING: DWORD = 3;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;
pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;
pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

// ===== FSCTL IOCTL Codes =====
// CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 0x0025, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const FSCTL_QUERY_USN_JOURNAL: DWORD = 0x00090094;
// CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 0x002B, METHOD_NEITHER, FILE_ANY_ACCESS)
pub const FSCTL_ENUM_USN_DATA: DWORD = 0x000900B0;
// CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 0x002F, METHOD_NEITHER, FILE_ANY_ACCESS)
pub const FSCTL_READ_USN_JOURNAL: DWORD = 0x000900BB;

// ===== USN Reason Flags =====
pub const USN_REASON_FILE_CREATE: DWORD = 0x00000100;
pub const USN_REASON_FILE_DELETE: DWORD = 0x00000200;
pub const USN_REASON_RENAME_NEW_NAME: DWORD = 0x00002000;
pub const USN_REASON_RENAME_OLD_NAME: DWORD = 0x00001000;
pub const USN_REASON_DATA_EXTEND: DWORD = 0x00000002;
pub const USN_REASON_DATA_OVERWRITE: DWORD = 0x00000001;
pub const USN_REASON_DATA_TRUNCATION: DWORD = 0x00000004;
pub const USN_REASON_CLOSE: DWORD = 0x80000000;

// ===== USN C Structures (repr(C)) =====

/// Returned by FSCTL_QUERY_USN_JOURNAL
#[repr(C)]
pub struct USN_JOURNAL_DATA_V0 {
    pub usn_journal_id: u64,  // DWORDLONG
    pub first_usn: i64,       // USN (INT64)
    pub next_usn: i64,        // USN (INT64)
    pub lowest_valid_usn: i64,// USN (INT64)
    pub max_usn: i64,         // USN (INT64)
    pub maximum_size: u64,    // DWORDLONG
    pub allocation_delta: u64,// DWORDLONG
}

/// Input for FSCTL_ENUM_USN_DATA
#[repr(C)]
pub struct MFT_ENUM_DATA_V0 {
    pub start_file_reference_number: u64, // DWORDLONG
    pub low_usn: i64,                     // USN (INT64)
    pub high_usn: i64,                    // USN (INT64)
}

/// Input for FSCTL_READ_USN_JOURNAL
#[repr(C)]
pub struct READ_USN_JOURNAL_DATA_V0 {
    pub start_usn: i64,        // USN (INT64)
    pub reason_mask: DWORD,
    pub return_only_on_close: DWORD,
    pub timeout: u64,          // DWORDLONG
    pub bytes_to_wait_for: u64,// DWORDLONG
    pub usn_journal_id: u64,   // DWORDLONG
}

// ===== Parsed USN Record (from variable-length USN_RECORD_V2) =====
#[derive(Debug, Clone)]
pub struct UsnRecord {
    pub record_length: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub file_reference_number: u64,
    pub parent_file_reference_number: u64,
    pub usn: i64,
    pub timestamp: i64,
    pub reason: u32,
    pub source_info: u32,
    pub security_id: u32,
    pub file_attributes: u32,
    pub file_name_length: u16,
    pub file_name_offset: u16,
    pub file_name: String,
}

/// Parse a USN_RECORD_V2 from raw bytes.
/// Returns None if the record is invalid.
pub fn parse_usn_record(data: &[u8]) -> Option<UsnRecord> {
    if data.len() < 60 { return None; } // Minimum USN_RECORD_V2 size

    let record_length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if record_length as usize > data.len() || record_length < 60 {
        return None;
    }

    let major_version = u16::from_le_bytes([data[4], data[5]]);
    let minor_version = u16::from_le_bytes([data[6], data[7]]);
    let file_reference_number = u64::from_le_bytes([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
    ]);
    let parent_file_reference_number = u64::from_le_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);
    let usn = i64::from_le_bytes([
        data[24], data[25], data[26], data[27],
        data[28], data[29], data[30], data[31],
    ]);
    let timestamp = i64::from_le_bytes([
        data[32], data[33], data[34], data[35],
        data[36], data[37], data[38], data[39],
    ]);
    let reason = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
    let source_info = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
    let security_id = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
    let file_attributes = u32::from_le_bytes([data[52], data[53], data[54], data[55]]);
    let file_name_length = u16::from_le_bytes([data[56], data[57]]);
    let file_name_offset = u16::from_le_bytes([data[58], data[59]]);

    // Parse file name (UTF-16LE)
    let file_name = if file_name_length > 0 && file_name_length as usize <= data.len().saturating_sub(file_name_offset as usize) {
        let name_start = file_name_offset as usize;
        let name_len = file_name_length as usize;
        let name_end = (name_start + name_len).min(data.len());
        let name_data = &data[name_start..name_end];
        // Convert UTF-16LE to String
        let u16_chars: Vec<u16> = name_data.chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&u16_chars)
    } else {
        String::new()
    };

    Some(UsnRecord {
        record_length,
        major_version,
        minor_version,
        file_reference_number,
        parent_file_reference_number,
        usn,
        timestamp,
        reason,
        source_info,
        security_id,
        file_attributes,
        file_name_length,
        file_name_offset,
        file_name,
    })
}


// ===== Token / Privilege Constants =====
pub const TOKEN_QUERY: DWORD = 0x0008;
pub const TOKEN_ADJUST_PRIVILEGES: DWORD = 0x0020;
pub const SE_PRIVILEGE_ENABLED: DWORD = 0x00000002;

// ===== Privilege Structures =====
#[repr(C)]
pub struct LUID {
    pub low_part: u32,
    pub high_part: i32,
}

#[repr(C)]
pub struct LUID_AND_ATTRIBUTES {
    pub luid: LUID,
    pub attributes: DWORD,
}

#[repr(C)]
pub struct TOKEN_PRIVILEGES {
    pub privilege_count: u32,
    pub privileges: [LUID_AND_ATTRIBUTES; 1],
}

// ===== Kernel32 FFI =====
#[link(name = "kernel32")]
extern "system" {
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    pub fn CloseHandle(hObject: HANDLE) -> BOOL;

    pub fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: LPCVOID,
        nInBufferSize: DWORD,
        lpOutBuffer: LPVOID,
        nOutBufferSize: DWORD,
        lpBytesReturned: *mut DWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;

    pub fn GetLastError() -> DWORD;

    pub fn OpenProcessToken(
        ProcessHandle: HANDLE,
        DesiredAccess: DWORD,
        TokenHandle: *mut HANDLE,
    ) -> BOOL;

    pub fn LookupPrivilegeValueW(
        lpSystemName: LPCWSTR,
        lpName: LPCWSTR,
        lpLuid: *mut LUID,
    ) -> BOOL;

    pub fn AdjustTokenPrivileges(
        TokenHandle: HANDLE,
        DisableAllPrivileges: BOOL,
        NewState: *const TOKEN_PRIVILEGES,
        BufferLength: DWORD,
        PreviousState: *mut TOKEN_PRIVILEGES,
        ReturnLength: *mut DWORD,
    ) -> BOOL;

    pub fn GetCurrentProcess() -> HANDLE;
}

// ===== Helper Functions =====

pub fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Convert a FILETIME (100-ns intervals since 1601-01-01) to Unix timestamp (seconds since 1970).
pub fn filetime_to_unix_epoch(filetime: i64) -> i64 {
    // FILETIME epoch: 1601-01-01
    // Unix epoch: 1970-01-01
    // Difference: 11644473600 seconds
    // FILETIME is in 100-ns intervals
    let unix_epoch_diff: i64 = 11644473600;
    let seconds = filetime / 10_000_000;
    seconds - unix_epoch_diff
}


