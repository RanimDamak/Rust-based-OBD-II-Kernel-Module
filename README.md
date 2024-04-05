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
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::{file, miscdev};
use kernel::prelude::*;
use kernel::sync::{Arc, ArcBorrow};
use kernel::str::CString;

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
    obd2_frame: Obd2Frame,
}

impl Device {

    fn new() -> Self {
        let mut vec = Vec::new();
        vec.try_push(0x11);
        vec.try_push(0x0D);
        let obd2_frame = Obd2Frame::new_request(
            2,
            1,
            0x1,
            vec,
        );
        Device {
            obd2_frame: obd2_frame,
        }
    }

    fn get_obd2_frame(&self) -> &Obd2Frame {
        &self.obd2_frame
    }

}

#[vtable]

impl file::Operations for Scull{
    type OpenData = Arc<Device>;
    type Data = Arc<Device>;

    fn open(context: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        let obd2_frame = context.get_obd2_frame();
        pr_info!("File for device {} was opened\n", obd2_frame.get_pid());
        Ok(context.clone())
    }

    fn read(
        _data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File was read\n");
        Ok(0)
    }

    fn write(
        _data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File was written\n");
        Ok(_reader.len())
    }
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello world\n");
      
        let dev = Arc::try_new(Device::new())?;

        let reg = miscdev::Registration::new_pinned(fmt!("scull"), dev)?;
        Ok(Scull{ _dev: reg })
    }
}

impl Obd2Frame {

     fn new_request(
        length: u8,
        mode: u8,
        pid: u8,
        data: Vec<u8>,
      
    ) -> Self {

        Obd2Frame {
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


    fn get_speed(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 {
            //let value = data[0];
            (data[0] * 10) as u16
        } else {
            0
        }

    }

    fn get_rpm(&self) -> u32 {
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
            
        } else {
            "Invalid fuel system status"
        }
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

    /*pub fn serialize(&self) -> CString {
        let mut serialized_frame = CString::try_from_fmt(fmt!("{:02x}{:02x}{:02x}"),self.length, self.mode, self.pid).unwrap();
        let mut output = CString::try_from_fmt(fmt!("{}"));
    
        for &byte in &self.data {
            output.try_push(CString::try_from_fmt(fmt!("{:02x}\n", byte)));
            
        }
        
        serialized_frame.from_vec(output).unwrap()
        //serialized_frame.copy_from_slice(output.as_slice()).unwrap()
        /*serialized_frame.clone_from_slice(output);
        serialized_frame*/
    }
    
*/
  
    pub fn serialize(&self) -> CString {
        let mut serialized_frame = CString::try_from_fmt(fmt!("")).unwrap();

        for value in [self.length, self.mode, self.pid] {
            let new_value = CString::try_from_fmt(fmt!("{:02x}\n", value)).unwrap();
            serialized_frame = [serialized_frame, new_value].concat();
        }

        for &byte in &self.data {
            let new_byte = CString::try_from_fmt(fmt!("{:02x}\n", byte)).unwrap();
            serialized_frame = [serialized_frame, new_byte].concat();
        }

        serialized_frame
    }
            
    


}


```


3. Testing:


```

pub fn serialize(&self) -> [u8; MAX_FRAME_SIZE] {
    let mut serialized_frame = [0u8; MAX_FRAME_SIZE];
    let mut index = 0;

    // Serialize length, mode, and pid
    for &value in &[self.length, self.mode, self.pid] {
        serialized_frame[index] = value;
        index += 1;
    }

    // Serialize data
    for &byte in &self.data {
        serialized_frame[index] = byte;
        index += 1;
    }

    serialized_frame
}


```
4. Testing2.0:


```
pub fn serialize(&self) -> CString {
       
        let mut serialized_frame = CString::try_from_fmt(fmt!("")).unwrap();

        // Serialize length, mode, and pid        
        for value in [self.length, self.mode, self.pid] {
            serialized_frame.try_push(CString::try_from_fmt(fmt!("{:02x}\n", value))).unwrap();
        }

        // Serialize data
        for &byte in &self.data {
            serialized_frame.try_push(CString::try_from_fmt(fmt!("{:02x}\n", byte))).unwrap();
        }

        serialized_frame
        
    }
```
V2.0:
```
pub fn serialize(&self) -> CString {
    let mut serialized_frame = CString::new("").unwrap();

    // Serialize length, mode, and pid
    for &value in &[self.length, self.mode, self.pid] {
        serialized_frame = CString::new(format!("{}{:02x}", serialized_frame.to_str().unwrap(), value)).unwrap();
    }

    // Serialize data
    for &byte in &self.data {
        serialized_frame = CString::new(format!("{}{:02x}", serialized_frame.to_str().unwrap(), byte)).unwrap();
    }

    serialized_frame
}


```
