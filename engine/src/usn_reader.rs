//! USN Journal reader for NTFS volumes.
//! Provides fast file enumeration using the NTFS USN Journal API,
//! replacing the slow recursive FsWalker with a 1-2 second full-volume scan.

use std::collections::HashMap;
use crate::ffi::*;
use crate::types::FileEntry;

/// Result of reading changes from USN Journal.
#[derive(Debug, Clone)]
pub enum FileChange {
    Added(FileEntry),
    Deleted(String),
    Modified(FileEntry),
    Renamed(String, String),
}

/// USN Journal reader leveraging NTFS USN Journal API.
pub struct UsnReader;

impl UsnReader {
    pub fn enumerate_volume(volume_letter: char) -> Result<Vec<FileEntry>, String> {
        let volume_path = format!("\\\\.\\{}:", volume_letter);
        let handle = open_volume(&volume_path)?;
        let journal_data = query_journal(handle)?;
        let journal_id = journal_data.usn_journal_id;
        let next_usn = journal_data.next_usn;
        let records = read_all_records(handle, journal_id, next_usn)?;

        if records.is_empty() {
            let _ = close_volume(handle);
            return Ok(Vec::new());
        }

        let mut path_map: HashMap<u64, (String, u64, bool, u32)> = HashMap::new();
        path_map.insert(5, (format!("{}:\\", volume_letter), 0, true, FILE_ATTRIBUTE_DIRECTORY));
        for record in &records {
            if !record.file_name.is_empty()
                && record.file_name != "$"
                && !record.file_name.starts_with('$')
                && record.file_name != "."
                && record.file_name != ".."
            {
                let is_dir = (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
                path_map.insert(record.file_reference_number,
                    (record.file_name.clone(), record.parent_file_reference_number, is_dir, record.file_attributes));
            }
        }

        let mut entries = Vec::new();
        let mut path_cache: HashMap<u64, String> = HashMap::new();
        path_cache.insert(5, format!("{}:\\", volume_letter));

        let dir_refs: Vec<u64> = path_map.iter()
            .filter(|(_, (_, _, is_dir, _))| *is_dir)
            .map(|(ref_num, _)| *ref_num)
            .collect();
        for ref_num in &dir_refs {
            resolve_path(*ref_num, &path_map, &mut path_cache, &volume_letter);
        }

        for record in &records {
            if record.file_name.is_empty()
                || record.file_name.starts_with('$')
                || record.file_name == "."
                || record.file_name == ".."
            {
                continue;
            }

            if let Some(full_path) = resolve_path(record.file_reference_number, &path_map, &mut path_cache, &volume_letter) {
                let is_dir = (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
                let is_hidden = (record.file_attributes & FILE_ATTRIBUTE_HIDDEN) != 0;
                let is_readonly = (record.file_attributes & FILE_ATTRIBUTE_READONLY) != 0;
                let parent_path = path_cache.get(&record.parent_file_reference_number)
                    .cloned()
                    .unwrap_or_else(|| format!("{}:\\", volume_letter));
                let extension = if is_dir {
                    String::new()
                } else {
                    std::path::Path::new(&record.file_name)
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default()
                };
                let modified = filetime_to_unix_epoch(record.timestamp);

                entries.push(FileEntry {
                    id: 0,
                    name: record.file_name.clone(),
                    path: full_path,
                    parent_path,
                    size: 0,
                    created: 0,
                    modified,
                    accessed: 0,
                    is_dir,
                    is_hidden,
                    is_readonly,
                    extension,
                    volume: volume_letter.to_string(),
                });
            }
        }

        close_volume(handle)?;
        Ok(entries)
    }

    pub fn read_changes(volume_letter: char, last_usn: i64) -> Result<Vec<FileChange>, String> {
        let volume_path = format!("\\\\.\\{}:", volume_letter);
        let handle = open_volume(&volume_path)?;
        let journal_data = query_journal(handle)?;
        let journal_id = journal_data.usn_journal_id;
        let current_usn = journal_data.next_usn;

        if last_usn >= current_usn {
            close_volume(handle)?;
            return Ok(Vec::new());
        }

        let mut read_data = READ_USN_JOURNAL_DATA_V0 {
            start_usn: last_usn,
            reason_mask: 0xFFFFFFFF,
            return_only_on_close: 0,
            timeout: 0,
            bytes_to_wait_for: 0,
            usn_journal_id: journal_id,
        };

        let mut changes = Vec::new();
        let mut buffer = vec![0u8; 65536];

        loop {
            let mut bytes_returned: DWORD = 0;
            let result = unsafe {
                DeviceIoControl(
                    handle,
                    FSCTL_READ_USN_JOURNAL,
                    &read_data as *const _ as LPCVOID,
                    std::mem::size_of::<READ_USN_JOURNAL_DATA_V0>() as DWORD,
                    buffer.as_mut_ptr() as LPVOID,
                    buffer.len() as DWORD,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                )
            };

            if result == 0 {
                break;
            }

            if bytes_returned < std::mem::size_of::<u32>() as DWORD {
                break;
            }

            let mut offset = std::mem::size_of::<u32>();
            while offset + 60 <= bytes_returned as usize {
                if let Some(record) = parse_usn_record(&buffer[offset..]) {
                    let reason = record.reason;
                    if reason == USN_REASON_CLOSE {
                        offset += record.record_length as usize;
                        continue;
                    }

                    let is_create = (reason & USN_REASON_FILE_CREATE) != 0;
                    let is_delete = (reason & USN_REASON_FILE_DELETE) != 0;
                    let is_rename_new = (reason & USN_REASON_RENAME_NEW_NAME) != 0;
                    let is_rename_old = (reason & USN_REASON_RENAME_OLD_NAME) != 0;
                    let is_modified = (reason & (USN_REASON_DATA_EXTEND | USN_REASON_DATA_OVERWRITE | USN_REASON_DATA_TRUNCATION)) != 0;

                    if is_create || is_rename_new || is_modified {
                        let is_dir = (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
                        let is_hidden = (record.file_attributes & FILE_ATTRIBUTE_HIDDEN) != 0;
                        let is_readonly = (record.file_attributes & FILE_ATTRIBUTE_READONLY) != 0;
                        let extension = if is_dir {
                            String::new()
                        } else {
                            std::path::Path::new(&record.file_name)
                                .extension()
                                .map(|e| format!(".{}", e.to_string_lossy()))
                                .unwrap_or_default()
                        };
                        let modified = filetime_to_unix_epoch(record.timestamp);
                        let path = format!("{}:\\{}", volume_letter, record.file_name);

                        let entry = FileEntry {
                            id: 0,
                            name: record.file_name.clone(),
                            path: path.clone(),
                            parent_path: String::new(),
                            size: 0,
                            created: 0,
                            modified,
                            accessed: 0,
                            is_dir,
                            is_hidden,
                            is_readonly,
                            extension,
                            volume: volume_letter.to_string(),
                        };

                        if is_create {
                            changes.push(FileChange::Added(entry));
                        } else if is_rename_new {
                            changes.push(FileChange::Added(entry));
                        } else {
                            changes.push(FileChange::Modified(entry));
                        }
                    }

                    if is_delete {
                        changes.push(FileChange::Deleted(record.file_name.clone()));
                    }
                    if is_rename_old {
                        changes.push(FileChange::Deleted(record.file_name.clone()));
                    }

                    offset += record.record_length as usize;
                } else {
                    offset += 4;
                }
            }

            let mut last_record_usn = read_data.start_usn;
            let mut off = std::mem::size_of::<u32>();
            while off + 60 <= bytes_returned as usize {
                if let Some(rec) = parse_usn_record(&buffer[off..]) {
                    last_record_usn = rec.usn;
                    off += rec.record_length as usize;
                } else {
                    off += 4;
                }
            }
            read_data.start_usn = last_record_usn + 1;
        }

        close_volume(handle)?;
        Ok(changes)
    }

    pub fn query_journal_id(volume_letter: char) -> Result<(i64, u64, u64), String> {
        let volume_path = format!("\\\\.\\{}:", volume_letter);
        let handle = open_volume(&volume_path)?;
        let journal_data = query_journal(handle)?;
        let result = (journal_data.next_usn, journal_data.usn_journal_id, journal_data.max_usn as u64);
        close_volume(handle)?;
        Ok(result)
    }

    pub fn is_usn_available(volume_letter: char) -> bool {
        let volume_path = format!("\\\\.\\{}:", volume_letter);
        match open_volume(&volume_path) {
            Ok(handle) => {
                let result = query_journal(handle).is_ok();
                let _ = close_volume(handle);
                result
            }
            Err(_) => false,
        }
    }
}

fn open_volume(path: &str) -> Result<HANDLE, String> {
    let wpath = to_wstring(path);
    let handle = unsafe {
        CreateFileW(
            wpath.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            0,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let err = unsafe { GetLastError() };
        return Err(format!("Failed to open volume {}: error {}", path, err));
    }
    Ok(handle)
}

fn close_volume(handle: HANDLE) -> Result<(), String> {
    let result = unsafe { CloseHandle(handle) };
    if result == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("Failed to close volume handle: error {}", err));
    }
    Ok(())
}

fn query_journal(handle: HANDLE) -> Result<USN_JOURNAL_DATA_V0, String> {
    let mut journal_data: USN_JOURNAL_DATA_V0 = unsafe { std::mem::zeroed() };
    let mut bytes_returned: DWORD = 0;

    let result = unsafe {
        DeviceIoControl(
            handle,
            FSCTL_QUERY_USN_JOURNAL,
            std::ptr::null(),
            0,
            &mut journal_data as *mut _ as LPVOID,
            std::mem::size_of::<USN_JOURNAL_DATA_V0>() as DWORD,
            &mut bytes_returned,
            std::ptr::null_mut(),
        )
    };

    if result == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("FSCTL_QUERY_USN_JOURNAL failed: error {}", err));
    }

    Ok(journal_data)
}

fn read_all_records(handle: HANDLE, _journal_id: u64, next_usn: i64) -> Result<Vec<UsnRecord>, String> {
    let mut all_records = Vec::new();
    let mut enum_data = MFT_ENUM_DATA_V0 {
        start_file_reference_number: 0,
        low_usn: 0,
        high_usn: next_usn,
    };

    let mut buffer = vec![0u8; 65536];

    loop {
        let mut bytes_returned: DWORD = 0;
        let result = unsafe {
            DeviceIoControl(
                handle,
                FSCTL_ENUM_USN_DATA,
                &enum_data as *const _ as LPCVOID,
                std::mem::size_of::<MFT_ENUM_DATA_V0>() as DWORD,
                buffer.as_mut_ptr() as LPVOID,
                buffer.len() as DWORD,
                &mut bytes_returned,
                std::ptr::null_mut(),
            )
        };

        if result == 0 {
            break;
        }

        if bytes_returned < std::mem::size_of::<u32>() as DWORD {
            break;
        }

        let mut offset = std::mem::size_of::<u32>();
        while offset + 60 <= bytes_returned as usize {
            if let Some(record) = parse_usn_record(&buffer[offset..]) {
                let rec_len = record.record_length as usize;
                enum_data.start_file_reference_number = record.file_reference_number + 1;
                all_records.push(record);
                offset += rec_len;
            } else {
                offset += 4;
            }
        }
    }

    Ok(all_records)
}

fn resolve_path(
    ref_num: u64,
    path_map: &HashMap<u64, (String, u64, bool, u32)>,
    path_cache: &mut HashMap<u64, String>,
    volume_letter: &char,
) -> Option<String> {
    if let Some(path) = path_cache.get(&ref_num) {
        return Some(path.clone());
    }

    let (name, parent_ref, _, _) = path_map.get(&ref_num)?;

    if ref_num == 5 {
        let root_path = format!("{}:\\", volume_letter);
        path_cache.insert(5, root_path.clone());
        return Some(root_path);
    }

    let parent_path = resolve_path(*parent_ref, path_map, path_cache, volume_letter)?;
    let full_path = if parent_path.ends_with('\\') {
        format!("{}{}", parent_path, name)
    } else {
        format!("{}\\{}", parent_path, name)
    };

    path_cache.insert(ref_num, full_path.clone());
    Some(full_path)
}



