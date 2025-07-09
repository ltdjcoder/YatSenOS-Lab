use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::drivers::serial::get_serial;

type Key = u8;


const INPUT_BUFFER_SIZE: usize = 128;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(INPUT_BUFFER_SIZE);
}


#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}


#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}


pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
        // core::hint::spin_loop();
    }
}


pub fn get_line() -> String {
    let mut line = String::with_capacity(INPUT_BUFFER_SIZE);
    
    loop {
        let key = pop_key();
        
        match key {
            b'\n' => {
                if let Some(mut serial) = get_serial() {
                    serial.send(b'\n');
                }
                break;
            }
            
            
            b'\r' => {
                if let Some(mut serial) = get_serial() {
                    serial.send(b'\n');
                }
                break;
            }
            
            // 退格键 (Backspace: 0x08, Delete: 0x7F)
            0x08 | 0x7F => {
                if !line.is_empty() {
                    line.pop();
                    
                    if let Some(mut serial) = get_serial() {
                        serial.backspace();
                    }
                }
            }
            
            key if key >= 32 && key <= 126 => {
                line.push(key as char);
                
                if let Some(mut serial) = get_serial() {
                    serial.send(key);
                }
            }
            
            _ => {
                
            }
        }
    }
    
    line
}


pub fn clear_buffer() {
    while try_pop_key().is_some() {
    }
}


pub fn buffer_len() -> usize {
    INPUT_BUF.len()
}


pub fn is_buffer_empty() -> bool {
    INPUT_BUF.is_empty()
}


pub fn is_buffer_full() -> bool {
    INPUT_BUF.is_full()
}
