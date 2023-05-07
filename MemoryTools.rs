use libc::{c_char, c_int,self, c_void, iovec, pid_t, process_vm_readv, process_vm_writev};
use std::ptr;
use std::{thread, time};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::io;
use std::fs;
use std::io::prelude::*;
use std::io::{Error, Result};
use std::os::unix::io::RawFd;





pub fn get_module_base(pid: i32, name: &str, index: i32) -> u64 {
    let mut i = 0;
    let mut start = 0;
    let mut end = 0;
    let fname = format!("/proc/{}/maps", pid);
    let file = match File::open(&fname) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file {}: {}", fname, e);
            return 0;
        }
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains(name) {
                i += 1;
                if i == index {
                    let mut parts = line.split('-');
                    if let Some(start_str) = parts.next() {
                        if let Some(end_str) = parts.next() {
                            let end_str = end_str.chars().filter(|c| c.is_digit(16)).collect::<String>();
                            println!("Parsing start address: {}", start_str);
                            println!("Parsing end address: {}", end_str);
                            start = match u64::from_str_radix(start_str, 16) {
                                Ok(start) => start,
                                Err(e) => {
                                    eprintln!("Failed to parse start address {}: {}", start_str, e);
                                    return 0;
                                }
                            };
                            end = match u64::from_str_radix(&end_str, 16) {
                                Ok(end) => end,
                                Err(e) => {
                                    eprintln!("Failed to parse end address {}: {}", end_str, e);
                                    return 0;
                                }
                            };
                            break;
                        }
                    }
                }
            }
        }
    }
    start
}





pub fn get_pid(package_name: &str) -> i32 {
    let dir = fs::read_dir("/proc").unwrap();
    for entry in dir {
        if let Ok(entry) = entry {
            let pid_str = entry.file_name().into_string().unwrap();
            if let Ok(pid) = pid_str.parse::<i32>() {
                let cmdline_path = format!("/proc/{}/cmdline", pid);
                if let Ok(file) = fs::File::open(cmdline_path) {
                    let mut reader = BufReader::new(file);
                    let mut cmdline = String::new();
                    reader.read_to_string(&mut cmdline).unwrap();
                    if cmdline.contains(package_name) {
                        return pid;
                    }
                }
            }
        }
    }
    return -1;
}


pub fn pwritev(pid: pid_t, address: libc::off_t, buffer: &[u8], size: usize) -> usize {
    let iov_write_buffer = iovec {
        iov_base: buffer.as_ptr() as *mut c_void,
        iov_len: size,
    };
    let mut iov_write_offset = iovec {
        iov_base: address as *mut c_void,
        iov_len: size,
    };
    let nwrite = unsafe {
        process_vm_writev(
            pid,
            &iov_write_buffer,
            1,
            &iov_write_offset,
            1,
            0,
        )
    };
    nwrite as usize
}

pub fn preadv(pid: pid_t, address: libc::off_t, buffer: &mut [u8], size: usize) -> usize {
    let mut iov_read_buffer = iovec {
        iov_base: buffer.as_mut_ptr() as *mut c_void,
        iov_len: size,
    };
    let iov_read_offset = iovec {
        iov_base: address as *mut c_void,
        iov_len: size,
    };
    let nread = unsafe {
        process_vm_readv(
            pid,
            &iov_read_buffer,
            1,
            &iov_read_offset,
            1,
            0,
        )
    };
    nread as usize
}



pub fn read_pointer_32(pid: RawFd, address: i64) -> i32 {
    let mut value: i32 = 0;
    let mut buffer = [0u8; 1024];
    preadv(pid, address,&mut buffer,std::mem::size_of::<i32>());
    value = i32::from_ne_bytes(buffer[0..std::mem::size_of::<i32>()].try_into().unwrap());
    return value;
}

pub fn read_pointer_64(pid: RawFd, address: i64) -> i64 {
    let mut value: i64 = 0;
    let mut buffer = [0u8; 1024];
    preadv(pid, address,&mut buffer,std::mem::size_of::<i64>());
    value = i64::from_ne_bytes(buffer[0..std::mem::size_of::<i64>()].try_into().unwrap());
    return value;
}




pub fn read_value<T: Copy>(pid: RawFd, address: i64) -> T {
    let mut buffer = [0u8; 1024];
    preadv(pid, address, &mut buffer, std::mem::size_of::<T>());
    let value = unsafe { std::ptr::read(buffer.as_ptr() as *const T) };
    value
}


pub fn Edit<T: Copy>(pid: RawFd, address: i64, new_value: T) {
    let bytes_to_write = std::mem::size_of::<T>();
    let value_bytes = unsafe {
        std::slice::from_raw_parts(
            &new_value as *const T as *const u8,
            bytes_to_write
        )
    };
    pwritev(pid, address, value_bytes, bytes_to_write);
}
