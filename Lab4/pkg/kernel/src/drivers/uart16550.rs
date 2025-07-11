use core::fmt;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    data: Port<u8>,                  // 数据寄存器 
    int_en: PortWriteOnly<u8>,       // 中断使能寄存器
    fifo_ctrl: PortWriteOnly<u8>,    // FIFO控制寄存器
    line_ctrl: PortWriteOnly<u8>,    // 线路控制寄存器
    line_sts: PortReadOnly<u8>,      // 线路状态寄存器
}

impl SerialPort {
    //fix-me
    pub const fn new(port: u16) -> Self {
        Self {
            data: Port::new(port),
            int_en: PortWriteOnly::new(port + 1),
            fifo_ctrl: PortWriteOnly::new(port + 2),
            line_ctrl: PortWriteOnly::new(port + 3),
            line_sts: PortReadOnly::new(port + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // 禁用所有中断
            self.int_en.write(0x00);
            
            // 设置波特率分频器
            self.line_ctrl.write(0x80);
            
            // 设置波特率为38400 (分频器=3)
            self.data.write(0x03);        // 低字节
            self.int_en.write(0x00);      // 高字节
            
            // 8位数据，无奇偶校验，1个停止位
            self.line_ctrl.write(0x03);
            
            // 启用FIFO
            self.fifo_ctrl.write(0xC7);
            
            
            // // 发送测试字节0xAE
            // self.data.write(0xAE);
            
            // // 检查串口是否正常工作
            // if self.data.read() != 0xAE {
            //     return; 
            // }

            self.int_en.write(0x01);   
        }
    }


    // fix-me
    pub fn send(&mut self, data: u8) {
        unsafe {
            // 等待发送缓冲区为空
            while (self.line_sts.read() & 0x20) == 0 {}
            
            // 发送数据
            self.data.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // fix-me
        unsafe {
            if (self.line_sts.read() & 1) != 0 {
                Some(self.data.read())
            } else {
                None
            }
        }
    }

    /// 发送退格序列：退格 + 空格 + 退格
    /// 这会删除终端上的最后一个字符
    pub fn backspace(&mut self) {
        self.send(0x08); // 退格
        self.send(0x20); // 空格（清除字符）
        self.send(0x08); // 再次退格
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
