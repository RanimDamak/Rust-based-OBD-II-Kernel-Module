# Testing

1. RUST
   
```
#[repr(C)]

#[derive(Debug)]
struct ObdFrame {
    length: u8,
    mode: u8,
    pid: u8,
    data: Vec<u8>,
}

impl ObdFrame {

    pub fn new_request(
        length: u8,
        mode: u8,
        pid: u8,
        data: Vec<u8>,
      
    ) -> Self {

        ObdFrame {
            length: length,
            mode: mode,
            pid: pid,
            data: data,
        }
        
    }
    
    fn get_length(&self) -> u8 {
        self.length
    }

    fn get_mode(&self) -> u8 {
        self.mode
    }

    fn get_pid(&self) -> u8 {

        self.pid
    }

    fn get_data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn get_speed(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 {
            (data[0] * 10) as u16
        } else {
            0
        }

    }


    pub fn get_rpm(&self) -> u32 {
        let data = self.get_data();
        if data.len() >= 2 {
            let a = data[0] as u16;
            let b = data[1] as u16;
            let rpm = (256 * a + b) as u32 / 4;
            rpm
        } else {
            0
        }
    }

    pub fn get_fuel_system_status(&self) -> String {
        let data = self.get_data();
        if data.len() >= 1 {
            let status = data[0];
            let status_str = match status {
                0x10 => "Closed loop, using oxygen sensor feedback for fuel mix",
                0x11 => "Open loop, using fixed values for fuel mix",
                0x12 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 1",
                0x13 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 2",
                0x14 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 1 and 2",
                0x15 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from short term fuel trim bank 1",
                0x16 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from short term fuel trim bank 2",
                0x17 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 1",
                0x18 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 2",
                0x19 => "Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 1 and 2",
                _ => "Invalid fuel system status",
            };
            status_str.to_string()
        } else {
            "Invalid fuel system status".to_string()
        }
    }

    pub fn get_data_dep_pid(&self) -> String {
        match self.get_pid() {
            //Vehicle Speed
            0x0D => {
                let speed = self.get_speed();
                format!("Vehicle Speed: {}", speed)
            }

            //RPM
            0x0C => {
                let rpm: u16 = self.get_rpm().try_into().unwrap();
                format!("RPM: {}", rpm)
            }

            //Fuel System Status
            0x01 => {
                let fuel_system_status = self.get_fuel_system_status();
                format!("Fuel System Status: {}", fuel_system_status)
            }

            //invalid
            _ => {
                println!("Invalid PID.");
                "Invalid PID".to_string()
            }
        }
    }

    pub fn serialize(&self) -> String {
        let mut serialized_frame = String::new();

        for value in [self.length, self.mode, self.pid] {
            serialized_frame.push_str(&format!("{:02x}",value));
        }

        for byte in &self.data {
            serialized_frame.push_str(&format!("{:02x}", byte));
        }
        
        serialized_frame
    }

}



fn main() {
    let frame = ObdFrame::new_request(
        2,
        1,
        0x0D,
        vec![0x0D, 0x0D],
    );

    let _length = frame.get_length();
    let _mode = frame.get_mode();
    let _pid = frame.get_pid();

    let _data = frame.get_data_dep_pid();

    println!("Data: {}\n", _data);
    let data_serialized=frame.serialize();
    println!("Serialized Data:{}\n",data_serialized);

}

```

2. RUST IN KERNEL ENVIRONNEMENT
   
```

//! This crate provides implementations for handling input/output buffers, file operations,
//! synchronization primitives, and string types in the kernel context.

use kernel::{
    io_buffer::{IoBufferReader, IoBufferWriter}, //readers and writers for input/output buffers
    {file, miscdev}, //modules related to handling files and miscellaneous devices
    prelude::*, //module that brings common traits and types into scope
    sync::{smutex::Mutex, Arc, ArcBorrow}, //synchronization primitives like mutex and atomic reference counting
    str::{CString,CStr}, //string types for handling C-style strings
    file::flags,
};
use alloc::vec::Vec;
use core::clone::Clone;

module! {
    type: Scull,
    name: "scull_test",
    license: "GPL",
}

struct Obd2Frame {
    length: u8,
    mode: u8,
    pid: u8,
    data: Vec<u8>,
} 

struct Scull {
    _dev: Pin<Box<miscdev::Registration<Scull>>>,
}

struct Device {
    contents: Mutex<Vec<u8>>,
}

impl Device {

    fn new(_obd2_frame: Obd2Frame) -> Self {
        
        Device {
            contents: Mutex::new(Vec::new()),
        }
    }

}

#[vtable]

impl file::Operations for Scull{

    type OpenData = Arc<Device>;
    type Data = Arc<Device>;

    fn open(context: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {

        pr_info!("File for device was opened\n");
        if _file.flags() & flags::O_ACCMODE == flags::O_WRONLY {
            context.contents.lock().clear();
        }
        Ok(context.clone())
        
    }

    fn read(
        _data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {

        pr_info!("File was read\n");

        let _offset = _offset.try_into()?;
        let vec = _data.contents.lock();

        let len = core::cmp::min(_writer.len(), vec.len().saturating_sub(_offset));
        pr_info!("-------------------\n");
        _writer.write_slice(&vec[_offset..][..len])?;

        Ok(len)
        
    }

    fn write(
        _data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {

        pr_info!("File was written\n");

        let _offset = _offset.try_into()?;
        let len = _reader.len();
        let new_len = len.checked_add(_offset).ok_or(EINVAL)?;
        let mut vec = _data.contents.lock();

        if new_len > vec.len() {
            vec.try_resize(new_len, 0)?;
        }

        // Create the OBD2 frame
        let obd2_frame = Obd2Frame{
            length: 0x03,
            mode: 0x01,
            pid: 0x0D,
            data: Vec::new(),
        };

        // Append OBD headers to the data
        vec.try_push(obd2_frame.length)?;
        vec.try_push(obd2_frame.mode)?;
        vec.try_push(obd2_frame.pid)?;

        // Append data to the buffer
        vec.try_extend_from_slice(&obd2_frame.data)?;

        Ok(_reader.len())

    }
}

impl kernel::Module for Scull {

    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {

        pr_info!("Hello world!\n");
        pr_info!("-----------------------\n");
        pr_info!("starting device!\n");
        pr_info!("watching for changes...\n");
        pr_info!("-----------------------\n");
        let dev = Arc::try_new(Device{ contents: Mutex::new(Vec::new())})?;
        let reg = miscdev::Registration::new_pinned(fmt!("scull_test"), dev)?;
        Ok(Scull{ _dev: reg })

    }

}

impl Obd2Frame {

    fn new_request(length: u8, mode: u8, pid: u8, data: Vec<u8>) -> Self {

        Obd2Frame {
            length,
            mode,
            pid,
            data,
        }
        
    }

    fn get_length(&self) -> u8 { self.length }

    fn get_mode(&self) -> u8 { self.mode }

    fn get_pid(&self) -> u8 { self.pid }

    fn get_data(&self) -> &[u8] { &self.data[..] }

    fn get_speed(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { (data[0] * 10) as u16 } 
        else { 0 }

    }

    fn get_rpm(&self) -> u32 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let rpm = (256 * a + b) as u32 / 4;
            rpm

        } 
        else { 0 }

    }

    fn get_fuel_system_status(&self) -> &str {

        let data = self.get_data();

        if data.len() >= 1 {

            let status = data[0];

            let status_str = match status {
                0x10 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix",
                0x11 => "Fuel System Status: Open loop, using fixed values for fuel mix",
                0x12 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 1",
                0x13 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 2",
                0x14 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term fuel trim bank 1 and 2",
                0x15 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from short term fuel trim bank 1",
                0x16 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from short term fuel trim bank 2",
                0x17 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 1",
                0x18 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 2",
                0x19 => "Fuel System Status: Closed loop, using oxygen sensor feedback for fuel mix, with valid data from long term and short term fuel trim bank 1 and 2",
                _ => "Invalid fuel system status",
            };

            status_str
            
        } 
        else { "Invalid fuel system status" }

    }

    fn get_data_dep_pid(&self) -> CString {

        match self.get_pid() {

            //Vehicle Speed
            0x0D => {
                let speed = self.get_speed();
                CString::try_from_fmt(fmt!("Vehicle Speed: {}", speed)).unwrap()
            }

            //RPM
            0x0C => {
                let rpm: u16 = self.get_rpm().try_into().unwrap();
                CString::try_from_fmt(fmt!("RPM: {}", rpm)).unwrap()
            }

            //Fuel System Status
            0x01 => {
                CString::try_from_fmt(fmt!("Fuel System Status: {}", self.get_fuel_system_status())).unwrap()
            }

            //invalid
            _ => {
                pr_info!("Invalid PID.");
                CString::try_from_fmt(fmt!("Invalid PID")).unwrap()
            }
            
        }

    }

    fn serialize(&self) -> &CStr {

        let binding: [u8; 3] = [self.length, self.mode, self.pid];
        let a: &[u8] = binding.as_slice();
        let b: &[u8] = self.data.as_slice();

        let c: &[u8] = {

            let mut v = Vec::try_with_capacity(a.len() + b.len()).unwrap();
            v.try_extend_from_slice(a).unwrap();       
            v.try_extend_from_slice(b).unwrap();       
            Box::leak(v.try_into_boxed_slice().unwrap())

        };

        let serialized_frame = CStr::from_bytes_with_nul(c).unwrap();
        serialized_frame
    
    }

}

```
