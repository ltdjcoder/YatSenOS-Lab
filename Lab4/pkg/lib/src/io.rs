use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string
        let mut line = String::new();
        let mut buf = [0u8; 1]; // 一次读取一个字节
        
        loop {
            // 从标准输入读取一个字节
            if let Some(bytes_read) = sys_read(0, &mut buf) {
                if bytes_read > 0 {
                    let ch = buf[0];
                    
                    match ch {
                        // 回车或换行表示行结束
                        b'\n' | b'\r' => {
                            break;
                        }
                        // 退格键处理
                        0x08 | 0x7F => {
                            if !line.is_empty() {
                                line.pop();
                            }
                        }
                        // 可打印字符
                        ch if ch >= 32 && ch <= 126 => {
                            line.push(ch as char);
                        }
                        // 忽略其他字符
                        _ => {}
                    }
                } else {
                    // 没有数据可读，继续等待
                    // 在实际系统中这里可能需要更智能的处理
                    continue;
                }
            } else {
                // 读取失败，跳出循环
                break;
            }
        }
        
        line
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
