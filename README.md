# Testing

0. function
```
fn ascii_to_decimal(ascii_code: u8) -> Result<u8, &'static str> {
    if ascii_code >= b'0' && ascii_code <= b'9' {
        Ok(ascii_code - b'0')
    } else {
        Err("Invalid ASCII code for a decimal digit")
    }
}

fn decimal_to_hex(decimal: u8) -> Result<[u8; 2], &'static str> {
    if decimal > 255 {
        return Err("Input out of range, should be between 0 and 255");
    }

    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let high_nibble = (decimal >> 4) & 0xF;
    let low_nibble = decimal & 0xF;

    Ok([HEX_DIGITS[high_nibble as usize], HEX_DIGITS[low_nibble as usize]])
}

fn main() {
    let decimal_value = 59; // Example decimal value

    match decimal_to_hex(decimal_value) {
        Ok(hex_digits) => {
            // Replace these lines with your kernel's print mechanism
            // Example: print the hexadecimal digits
            // print!("Hex value: {}{}", hex_digits[0] as char, hex_digits[1] as char); // Adjust according to your environment
        },
        Err(e) => {
            // Handle the error, e.g., print an error message
            // print!("Error: {}", e); // Adjust according to your environment
        },
    }
}

```


1. RUST
   
```

use std::convert::TryInto;
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

   fn decode_supported_pids(response: u32) -> [bool; 32] {
    let mut supp_pids = [false; 32];
    for i in 0..32 {
        let mask = 1 << (31 - i);
        supp_pids[i] = (response & mask) != 0;
    }
    supp_pids
}

fn get_supported_pids(&self) -> Vec<u8> {
    let data = Self::vec_to_u32(self.get_data());
    let pids = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20];
    let supported_bits = Self::decode_supported_pids(data);
    let mut v = Vec::new();

    for i in 0..32 {
        if supported_bits[i] {
            v.push(pids[i]);
        }
    }
    v
}

fn vec_to_u32(vec: &[u8]) -> u32 {
    let mut result: u32 = 0;
    for (i, &byte) in vec.iter().enumerate() {
        result |= (byte as u32) << (24 - i * 8);
    }
    result
}

    pub fn get_data_dep_pid(&self) -> String {
        match self.get_pid() {

            //Show PIDs supported
            0x00 => {
                format!("Supported PIDs: {:?}",self.get_supported_pids())
            }

            //Fuel System Status
            0x01 => {
                let fuel_system_status = self.get_fuel_system_status();
                format!("Fuel System Status: {}", fuel_system_status)
            }

            //RPM
            0x0C => {
                let rpm: u16 = self.get_rpm().try_into().unwrap();
                format!("RPM: {}", rpm)
            }

            //Vehicle Speed
            0x0D => {
                let speed = self.get_speed();
                format!("Vehicle Speed: {}", speed)
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
        3,
        1,
        0x00,
        vec![0x0D, 0x0D,0x0D, 0x0D],
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
**************************************************************************************************************************************************************************************************************************************************


2. Rust Echo Server
```

// SPDX-License-Identifier: GPL-2.0

//! Rust echo server sample.

use kernel::{
    kasync::executor::{workqueue::Executor as WqExecutor, AutoStopHandle, Executor},
    kasync::net::{TcpListener, TcpStream},
    net::{self, Ipv4Addr, SocketAddr, SocketAddrV4},
    prelude::*,
    spawn_task,
    sync::{Arc, ArcBorrow},
};

async fn echo_server(stream: TcpStream) -> Result {
    let mut buf = [0u8; 8];
    loop {
        let n = stream.read(&mut buf).await?;
        pr_info!("echoserverread") ;
        pr_info!("buffer is{:?}", buf) ; 
        pr_info!("------------------------------------") ; 
        if n == 0 {
            pr_info!("Not getting anything!");
            return Ok(());
        }
        stream.write_all(&buf[..n]).await?;
    }
}

async fn accept_loop(listener: TcpListener, executor: Arc<impl Executor>) {
    loop {
        if let Ok(stream) = listener.accept().await {
            pr_info!("Client is connected!");
            let _ = spawn_task!(executor.as_arc_borrow(), echo_server(stream));
        }
    }
}

fn start_listener(ex: ArcBorrow<'_, impl Executor + Send + Sync + 'static>) -> Result {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::ANY, 8080));
    let listener = TcpListener::try_new(net::init_ns(), &addr)?;
    spawn_task!(ex, accept_loop(listener, ex.into()))?;
    Ok(())
}

struct RustEchoServer {
    _handle: AutoStopHandle<dyn Executor>,
}

impl kernel::Module for RustEchoServer {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {

        pr_info!("Hello from Server!\n");
        pr_info!("-------------------------\n");
        pr_info!("Starting!\n");
        pr_info!("-------------------------\n");
        
        let handle = WqExecutor::try_new(kernel::workqueue::system())?;
        start_listener(handle.executor())?;
        Ok(Self {
            _handle: handle.into(),
        })
    }
}

module! {
    type: RustEchoServer,
    name: "rust_echo_server",
    author: "Rust for Linux Contributors",
    description: "Rust tcp echo sample",
    license: "GPL v2",
}



```

3. RUST scull
   
```

//! This crate provides implementations for handling input/output buffers, file operations,
//! synchronization primitives, and string types in the kernel context.

use kernel::{
    io_buffer::{IoBufferReader, IoBufferWriter},//readers and writers for input/output buffers
    {file, miscdev},                            //modules related to handling files and miscellaneous devices
    prelude::*,                                 //module that brings common traits and types into scope
    sync::{smutex::Mutex, Arc, ArcBorrow},      //synchronization primitives like mutex and atomic reference counting
    str::{CString,CStr},                        //string types for handling C-style strings
    file::{flags, Operations},
    net::{TcpStream, Ipv4Addr, SocketAddr, SocketAddrV4, IpProtocol, SockType, AddressFamily, Socket },
    
};
use alloc::{str::from_utf8, vec::Vec};
use core::clone::Clone;


/// Establishes a TCP connection to the specified address.
///
/// # Arguments
/// * `address` - The address to connect to.
///
/// # Returns
/// A `Result` containing a new `TcpStream` if the connection was successful, or an `Err` containing the error that occurred.

pub fn connect(address: &SocketAddr) -> Result<TcpStream> {
    // Create a new TCP socket.
    let socket = Socket::new(AddressFamily::Inet, SockType::Stream, IpProtocol::Tcp)?;
    
    // Print debug information.
    pr_info!("number1");
    
    // Connect the socket to the specified address.
    socket.connect(address, 0)?; 
    
    // Print debug information.
    pr_info!("number2") ;
    
    // Return a new TcpStream.
    Ok(TcpStream {sock:unsafe{socket.as_inner()}})
}

// pub fn send_number(stream: &TcpStream, number: u32) -> Result<usize> {
//     pr_info!("beginnings") ;
//     let number_bytes = number.to_le_bytes();
//     pr_info!("send number\n") ; 
//     stream.write(&number_bytes, true) 
// }


/// Sends data over a TCP stream.
///
/// # Arguments
/// * `stream` - The TCP stream to send data over.
/// * `data` - The data to send as a vector of bytes.
///
/// # Returns
/// A `Result` containing the number of bytes written if the data was sent successfully, or an `Err` containing the error that occurred.

pub fn send_data(stream: &TcpStream, data: Vec<u8>) -> Result<usize> {
    // Ensure the data vector has exactly 5 elements
    let mut buffer = [0u8; 5];
    for (i, &item) in data.iter().enumerate() {
        if i >= 5 {
            break; // Prevent index out of bounds
        }
        buffer[i] = item;
    }
    
    // Write the data vector to the stream
    stream.write(&buffer, true)

    // Return the number of bytes written
}

/// A simple TCP client for testing purposes.
///
/// This struct represents a simple TCP client for testing purposes. It contains a single field,
/// `stream`, which is a `TcpStream` struct representing the underlying TCP connection.

pub struct RustClientTest {
    stream: TcpStream,
}


module! {
    type: Scull,
    name: "scull_test",
    license: "GPL",
    params: {

        req_resp: i8 {
            default: 0,
            permissions: 0o000,
            description: "Resquest(0) or Response(1)",
        },

        _mode: u8 {
            default: 1,
            permissions: 0o000,
            description: "10 modes for resquest(0.) & 10 modes for Response(4.)",
        },

        // _pid: u8 {
        //     default: 13,
        //     permissions: 0o000,
        //     description: "Vehicule Speed(0x0D) or RPM(0x0C) or Fuel System Status (0x01)",
        // },

    },
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

        pr_info!("File was opened\n");
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
        pr_info!("---------------------\n");
        _writer.write_slice(&vec[_offset..][..len])?;


        Ok(len)
        
    }

    fn write(
        _data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {

        let remote_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::ANY, 8080)) ;
        pr_info!("number3") ; 
        let stream = connect(&remote_addr)?;
        pr_info!("number4") ;
        

        let mut data : Vec<u8> = Vec::new() ; 
        for _i in 0..5 {
            let _ = data.try_push(1) ;
        }

        send_data(&stream, data)? ;

        //let stream = TcpStream(sock);
        let slice= &[0 as u8; 8];           //8-byte slice 
        pr_info!("---------------------\n");
        //write data to the stream
        stream.write(slice,true).unwrap();
        pr_info!("Client: OK sent!\n");
        let msg= b"Hello!";
        let mut data=[0 as u8;6];           //6 byte buffer
        match stream.read(&mut data, true) {
            Ok(_) => {
                if &data == msg {
                    pr_info!("Reply is OK!");
                } else {
                    let text = from_utf8(&data).unwrap();
                    pr_info!("Unexpected reply: {}",text);
                }
            },
            Err(e) => {
                pr_info!("Failed to recieve data");
            }
        }


        let _offset = _offset.try_into()?;
        let len = _reader.len();
        let new_len = len.checked_add(_offset).ok_or(EINVAL)?;
        let mut vec = _data.contents.lock();

        if new_len > vec.len() {
            vec.try_resize(new_len, 0)?;
        }

        // Create the OBD2 frame
        let obd2_frame = Obd2Frame{
            length: 3, // 0x03
            mode: 1, // 0x01
            pid: 13, // 0x0D
            data: Vec::from(_reader.read_all()?),
        };

        // Append OBD headers to the data
        vec.try_push(obd2_frame.length)?;
        vec.try_push(obd2_frame.mode)?;
        vec.try_push(obd2_frame.pid)?;
        pr_info!("element: {}\n", &vec[0]);

        // Append data to the buffer
        for elt in &obd2_frame.data {
            vec.try_push(*elt)?;
            pr_info!("element: {}\n", *elt);
        }
        
        //obd2_frame.get_data_dep_pid();

        Ok(len)

    }
}

impl kernel::Module for Scull {

    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {

        pr_info!("Hello from Client!\n");
        pr_info!("-------------------------\n");
        pr_info!("Starting device!\n");
        pr_info!("waiting for changes.....\n");
        pr_info!("-------------------------\n");

        pr_info!("Rust scull module parameters sample (init)\n");

        {
            let _lock = _module.kernel_param_lock();
            pr_info!("Parameters:\n");
            pr_info!("Resquest(0) or Response(1): {}\n", req_resp.read());
            pr_info!("Mode: {}\n", _mode.read());
            //pr_info!("PID: {}\n", _pid.read());
        }

        let dev = Arc::try_new(Device{ contents: Mutex::new(Vec::new())})?;
        let reg = miscdev::Registration::new_pinned(fmt!("scull_test"), dev)?;

        let remote_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::ANY, 8080)) ;
        pr_info!("number3") ; 
        let stream = connect(&remote_addr)?;
        pr_info!("number4") ;
        // // Example number to send
        // let number_to_send = 42;
        // send_number(&stream, number_to_send)? ; 

        let mut data : Vec<u8> = Vec::new() ; 
        for _i in 0..5 {
            let _ = data.try_push(1) ;
        }

        send_data(&stream, data)? ; 

        Ok(Scull{ _dev: reg })

    }

}

// impl Drop for Scull {
//     fn drop(&mut self) {
//         pr_info!("Rust Scull module parameters sample (exit)\n");
//     }
// }

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

    // fn get_modes_from_mode(&self){
        
    //     m = self.get_mode();

    //     if (req_resp= 0) {

    //         match m {

    //             0x01 =>  ,
    //             0x03 => ,
    //             0x05 => ,
    //             0x09 => ,
    //             _ => "Invalid mode",
    
    //         };

    //     }
    //     else if (req_resp= 1) {

    //         match m {

    //             0x41 => self.get_data_from_pid(),
    //             0x43 => ,
    //             0x45 => ,
    //             0x49 => ,
    //             _ => "Invalid mode",
    
    //         };
            
    //     }
        

    // }

   


    fn get_rpm(&self) -> u32 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let rpm = (256 * a + b) as u32 / 4 ;
            rpm

        } 
        else { 0 }

    }

    fn get_fuel_system_status(&self) -> &str {

        let data = self.get_data();

        if data.len() >= 1 {

            let status = data[0];

            let status_str = match status {
                0x00 => "Fuel System Status: The motor is off",
                0x01 => "Fuel System Status: Open loop due to insufficient engine temperature",
                0x02 => "Fuel System Status: Closed loop, using oxygen sensor feedback to determine fuel mix",
                0x04 => "Fuel System Status: Open loop due to engine load OR fuel cut due to deceleration",
                0x08 => "Fuel System Status: Open loop due to system failure",
                0x10 => "Fuel System Status: Closed loop, using at least one oxygen sensor but there is a fault in the feedback system",
                _ => "Invalid fuel system status",
            };

            status_str
            
        } 
        else { "Invalid fuel system status" }

    }

    

    fn get_commanded2air_status(&self) -> &str {

        let data = self.get_data();

        if data.len() >= 1 {

            let status = data[0];

            let status_str = match status {
                
                0x01 => "Upstream",
                0x02 => "Downstream of catalytic converter",
                0x04 => "From the outside atmosphere or off",
                0x08 => "Pump commanded on for diagnostics",
                
                _ => "Invalid Commanded secondary air status",
            };

            status_str
            
        } 
        else { "Invalid Commanded secondary air status"}

    }

    fn get_obd_standards(&self) -> &str {
        let data = self.get_data();
    
        if data.len() >= 1 {
            let standards = data[0];
    
            let standards_str = match standards {
                1 => "OBD-II as defined by the CARB",
                2 => "OBD as defined by the EPA",
                3 => "OBD and OBD-II",
                4 => "OBD-I",
                5 => "Not OBD compliant",
                6 => "EOBD (Europe)",
                7 => "EOBD and OBD-II",
                8 => "EOBD and OBD",
                9 => "EOBD, OBD and OBD II",
                10 => "JOBD (Japan)",
                11 => "JOBD and OBD II",
                12 => "JOBD and EOBD",
                13 => "JOBD, EOBD, and OBD II",
                17 => "Engine Manufacturer Diagnostics (EMD)",
                18 => "Engine Manufacturer Diagnostics Enhanced (EMD+)",
                19 => "Heavy Duty On-Board Diagnostics (Child/Partial) (HD OBD-C)",
                20 => "Heavy Duty On-Board Diagnostics (HD OBD)",
                21 => "World Wide Harmonized OBD (WWH OBD)",
                23 => "Heavy Duty Euro OBD Stage I without NOx control (HD EOBD-I)",
                24 => "Heavy Duty Euro OBD Stage I with NOx control (HD EOBD-I N)",
                25 => "Heavy Duty Euro OBD Stage II without NOx control (HD EOBD-II)",
                26 => "Heavy Duty Euro OBD Stage II with NOx control (HD EOBD-II N)",
                28 => "Brazil OBD Phase 1 (OBDBr-1)",
                29 => "Brazil OBD Phase 2 (OBDBr-2)",
                30 => "Korean OBD (KOBD)",
                31 => "India OBD I (IOBD I)",
                32 => "India OBD II (IOBD II)",
                33 => "Heavy Duty Euro OBD Stage VI (HD EOBD-IV)",
                _ => "Unknown OBD standard",
            };
    
            standards_str
        } else {
            "Invalid response from OBD device"
        }
    }

    fn get_fuel_type_coding(&self) -> &str {
        let data = self.get_data();
    
        if data.len() >= 1 {
            let fuel_type = data[0];
    
            let fuel_type_str = match fuel_type {
                0 => "Not available",
                1 => "Gasoline",
                2 => "Methanol",
                3 => "Ethanol",
                4 => "Diesel",
                5 => "LPG",
                6 => "CNG",
                7 => "Propane",
                8 => "Electric",
                9 => "Bifuel running Gasoline",
                10 => "Bifuel running Methanol",
                11 => "Bifuel running Ethanol",
                12 => "Bifuel running LPG",
                13 => "Bifuel running CNG",
                14 => "Bifuel running Propane",
                15 => "Bifuel running Electricity",
                16 => "Bifuel running electric and combustion engine",
                17 => "Hybrid gasoline",
                18 => "Hybrid Ethanol",
                19 => "Hybrid Diesel",
                20 => "Hybrid Electric",
                21 => "Hybrid running electric and combustion engine",
                22 => "Hybrid Regenerative",
                23 => "Bifuel running diesel",
                _ => "Reserved by ISO/SAE",
            };
    
            fuel_type_str
        } else {
            "Invalid response from OBD device"
        }
    }

    fn vec_to_u32(vec: &[u8]) -> u32 {

        let mut result: u32 = 0;

        for (i, byte) in vec.iter().enumerate() {
            result |= (*byte as u32) << (24 - i * 8);
        }

        result

    }

    fn decode_supported_pids(response: u32) -> [bool; 32] {

        let mut supp_pids = [false; 32];

        for i in 0..32 {

            let mask = 1 << (31 - i);
            supp_pids[i] = (response & mask)!= 0;

        }

        supp_pids

    }

    fn get_supported_pids (&self) -> Vec<u32> {

        let data = Self::vec_to_u32(self.get_data());
        let pids= [0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F,0x20];
        let supported = Self::decode_supported_pids(data);
        let mut v = Vec::new();
    
        for i in 0..32 {

            if supported[i]==true{
                v.try_push(pids[i]).unwrap(); 
            }
            
        }

        v 

    }

    fn get_a2(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { (data[0]*100) as u16 /255 } 
        else { 0 }

    }

    fn get_a3(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { (data[0] - 125) as u16 } 
        else { 0 }

    }

    fn get_e(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 5 { (data[4] * 100 ) as u16 / 225  } 
        else { 0 }

    }

    fn get_d(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 4 { (data[3] * 100 ) as u16 / 225  } 
        else { 0 }

    }

    fn get_temperature(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { (data[0]-40) as u16 } 
        else { 0 }

    }

    fn get_fuel_pressure(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { (data[0]*3) as u16 } 
        else { 0 }

    }

    fn get_a(&self) -> u16 {

        let data = self.get_data();
        if data.len() >= 1 { data[0] as u16 } 
        else { 0 }

    }

    fn get_ab(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_ab2(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 /32 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_evap(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 / 200 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_fuel_rail(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) * 0.079 as u16 ;
            r.into()

        } 
        else { 0 }

    }


    fn get_fuel_rail_gauge(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) * 10 as u16 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_dpf(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) / 10 as u16 - 40 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_egr(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) - 100 as u16 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_pressure(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 * 10 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_fuel_injection_timing(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 / 128 - 210 ;
            r.into()

        } 
        else { 0 }

    }

    fn get_engine_fuel_rate(&self) -> u16 {

        let data = self.get_data();

        if data.len() >= 2 {

            let a = data[0] as u16;
            let b = data[1] as u16;
            let r = (256 * a + b) as u16 / 20 ;
            r.into()

        } 
        else { 0 }

    }
 
    fn get_data_from_pid(&self) -> CString {

        match self.get_pid() {

            //Show PIDs supported
            0x00 => {
                CString::try_from_fmt(fmt!("Supported PIDs: {:?}",self.get_supported_pids())).unwrap()
            }

            //Fuel System Status
            0x03 => {
                CString::try_from_fmt(fmt!("Fuel System Status: {}", self.get_fuel_system_status())).unwrap()
            }

            //Calculated engine load 
            0x04 => {
                CString::try_from_fmt(fmt!("Calculated engine load: {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //Engine coolant temperature 
            0x05 => {
                CString::try_from_fmt(fmt!("Engine coolant temperature: {}°C, Minimum Value: -40°C, Maximum Value: 215°C", self.get_temperature())).unwrap()
            }

            //Fuel pressure (gauge pressure) 
            0x0A => {
                CString::try_from_fmt(fmt!("Fuel pressure (gauge pressure): {} kPa, Minimum Value: 0 kPa, Maximum Value: 765 kPa",self.get_fuel_pressure())).unwrap()
            }

            //Intake manifold absolute pressure 
            0x0B => {
                CString::try_from_fmt(fmt!("Intake manifold absolute pressure: {} kPa, Minimum Value: 0 kPa, Maximum Value: 255 kPa", self.get_a())).unwrap()
            }

            //RPM
            0x0C => {
                let rpm: u16 = self.get_rpm().try_into().unwrap();
                CString::try_from_fmt(fmt!("RPM: {} rpm, Minimum Value: 0 rpm, Maximum Value: 16383.75 rpm", rpm)).unwrap()
            }

            //Vehicle Speed
            0x0D => {
                CString::try_from_fmt(fmt!("Vehicle Speed: {}km/h, Minimum Value: 0 km/h, Maximum Value:255km/h", self.get_a())).unwrap()
            }    

            //Intake air temperature 
            0x0F => {
                CString::try_from_fmt(fmt!("Intake air temperature: {}°C, Minimum Value: -40°C, Maximum Value: 215°C", self.get_temperature())).unwrap()
            }

            //Throttle position 
            0x11 => {
                CString::try_from_fmt(fmt!("Throttle position: {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            // Commanded secondary air status
            0x12 => {
                CString::try_from_fmt(fmt!("Commanded secondary air status: {}", self.get_commanded2air_status())).unwrap()
            }

            // OBD standards this vehicle conforms to
            0x1C => {
                CString::try_from_fmt(fmt!("OBD standards this vehicle conforms to: {}", self.get_obd_standards())).unwrap()
            }

            //Run time since engine start 
            0x1F => {
                CString::try_from_fmt(fmt!("Run time since engine start: {}s, Minimum Value: 0s, Maximum Value: 65,535s", self.get_ab())).unwrap()
            }

            //Distance traveled with malfunction indicator lamp (MIL) on 
            0x21 => {
                CString::try_from_fmt(fmt!("Distance traveled with malfunction indicator lamp (MIL) on: {}km, Minimum Value: 0km, Maximum Value: 65,535km", self.get_ab())).unwrap()
            }

            //Fuel Rail Pressure (relative to manifold vacuum) 
            0x22 => {
                CString::try_from_fmt(fmt!("Fuel Rail Pressure (relative to manifold vacuum): {} kPa, Minimum Value: 0 kPa, Maximum Value: 5177.265 kPa",self.get_fuel_rail() )).unwrap()
            }

            //Fuel Rail Gauge Pressure (diesel, or gasoline direct injection) 
            0x23 => {
                CString::try_from_fmt(fmt!("Fuel Rail Gauge Pressure (diesel, or gasoline direct injection): {} kPa, Minimum Value: 0 kPa, Maximum Value: 655,350 kPa",self.get_fuel_rail_gauge() )).unwrap()
            }

            //Commanded EGR
            0x2C => {
                CString::try_from_fmt(fmt!("Commanded EGR: {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //EGR Error 
            0x2D => {
                CString::try_from_fmt(fmt!("EGR Error: {}%, Minimum Value: -100%, Maximum Value: 99.2%", self.get_egr())).unwrap()
            }


            //Commanded evaporative purge
            0x2E => {
                CString::try_from_fmt(fmt!("Commanded evaporative purge: {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //Fuel Tank Level Input 
            0x2F => {
                CString::try_from_fmt(fmt!("Fuel Tank Level Input: {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //Warm-ups since codes cleared 
            0x30 => {
                CString::try_from_fmt(fmt!("Warm-ups since codes cleared: {}, Minimum Value: 0 , Maximum Value:255", self.get_a())).unwrap()
            }    

            //Distance traveled since codes cleared 
            0x31 => {
                CString::try_from_fmt(fmt!("Distance traveled since codes cleared: {}km, Minimum Value: 0km, Maximum Value: 65,535km", self.get_ab())).unwrap()
            }

            //Absolute Barometric Pressure  
            0x33 => {
                CString::try_from_fmt(fmt!("Absolute Barometric Pressure: {} kPa, Minimum Value: 0 kPa, Maximum Value:255 kPa
                ", self.get_a())).unwrap()
            }  

            //Relative throttle position
            0x45 => {
                CString::try_from_fmt(fmt!("Relative throttle position: {}%, Minimum Value: 0%, Maximum Value: 100%",self.get_a2() )).unwrap()
            }

            //Ambient air temperature 
            0x46 => {
                CString::try_from_fmt(fmt!("Ambient air temperature : {}°C, Minimum Value: -40°C, Maximum Value: 215°C", self.get_temperature())).unwrap()
            }

            //Fuel Type Coding
            0x51 => {
                CString::try_from_fmt(fmt!("Fuel Type Coding: {}", self.get_fuel_type_coding())).unwrap()
            }

            //Ethanol fuel % 
            0x52 => {
                CString::try_from_fmt(fmt!("Ethanol fuel % : {}%, Minimum Value: 0%, Maximum Value: 100%",self.get_a2() )).unwrap()
            }

            //Absolute Evap system Vapor Pressure 
            0x53 => {
                CString::try_from_fmt(fmt!("Absolute Evap system Vapor Pressure: {} kPa, Minimum Value: 0 kPa, Maximum Value: 327.675 kPa",self.get_evap() )).unwrap()
            }

            //Fuel rail absolute pressure  
            0x59 => {
                CString::try_from_fmt(fmt!("Fuel rail absolute pressure : {} kPa, Minimum Value: 0 kPa, Maximum Value: 655,350 kPa",self.get_pressure() )).unwrap()
            }

            //Relative accelerator pedal position 
            0x5A => {
                CString::try_from_fmt(fmt!("Relative accelerator pedal position : {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //Hybrid battery pack remaining life 
            0x5B => {
                CString::try_from_fmt(fmt!("Hybrid battery pack remaining life : {}%, Minimum Value: 0%, Maximum Value: 100%", self.get_a2())).unwrap()
            }

            //Engine oil temperature  
            0x5C => {
                CString::try_from_fmt(fmt!("Engine oil temperature: {}°C, Minimum Value: -210.00°, Maximum Value: 215°C", self.get_temperature())).unwrap()
            }

            //Fuel injection timing  
            0x5D => {
                CString::try_from_fmt(fmt!("Fuel injection timing: {}°, Minimum Value: -40°C, Maximum Value: 301.992°", self.get_fuel_injection_timing() )).unwrap()
            }

            // Engine fuel rate 
            0x5E => {
                CString::try_from_fmt(fmt!("Engine fuel rate: {} L/h , Minimum Value: 0 L/h , Maximum Value: 3212.75 L/h ", self.get_engine_fuel_rate())).unwrap()
            }

            //Driver's demand engine - percent torque 
            0x61 => {
                CString::try_from_fmt(fmt!("Driver's demand engine - percent torque : {}%, Minimum Value: -125%, Maximum Value: 130%", self.get_a3())).unwrap()
            }

            //Actual engine - percent torque 
            0x62 => {
                CString::try_from_fmt(fmt!("Actual engine - percent torque : {}%, Minimum Value: -125%, Maximum Value: 130%", self.get_a3())).unwrap()
            }

            // Diesel Particulate filter (DPF) temperature
            0x7C => {
                CString::try_from_fmt(fmt!("Diesel Particulate filter (DPF) temperature: {}°C", self.get_dpf())).unwrap()
            }

            // NOx reagent system
            0x85 => {
                CString::try_from_fmt(fmt!("NOx reagent system: {}%", self.get_e())).unwrap()
            }

            //Engine Friction - Percent Torque 
            0x8E => {
                CString::try_from_fmt(fmt!("Engine Friction - Percent Torque : {}%, Minimum Value: -125%, Maximum Value: 130%", self.get_a3())).unwrap()
            }

            //Diesel Exhaust Fluid Sensor Data 
            0x9B => {
                CString::try_from_fmt(fmt!("Diesel Exhaust Fluid Sensor Data  : {}%", self.get_d())).unwrap()
            }

            //Cylinder Fuel Rate 
            0xA2 => {
                CString::try_from_fmt(fmt!("Cylinder Fuel Rate : {} mg/stroke, Minimum Value: 0 mg/stroke, Maximum Value: mg/stroke", self.get_ab2())).unwrap()
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

4. Qemu:
```

~ # modinfo rust_scull
filename:       /lib/modules/6.3.0+/kernel/samples/rust/rust_scull.ko
license:        GPL
parm:           req_resp:Resquest(0) or Response(1)
parm:           _mode:10 modes for resquest(0.) & 10 modes for Response(4.)
depends:        
intree:         Y
vermagic:       6.3.0+ SMP mod_unload 
~ # modinfo rust_echo_server
filename:       /lib/modules/6.3.0+/kernel/samples/rust/rust_echo_server.ko
description:    Rust tcp echo sample
license:        GPL v2
depends:        
intree:         Y
vermagic:       6.3.0+ SMP mod_unload 
~ # modprobe rust_echo_server
[   30.103470] rust_echo_server: Hello from Server!
[   30.103684] rust_echo_server: -------------------------
[   30.103820] rust_echo_server: Starting!
[   30.103923] rust_echo_server: -------------------------
~ # modprobe rust_scull
[   48.527973] scull_test: Hello from Client!
[   48.528266] scull_test: -------------------------
[   48.528814] scull_test: Starting device!
[   48.528946] scull_test: waiting for changes.....
[   48.529105] scull_test: -------------------------
[   48.529231] scull_test: Rust scull module parameters sample (init)
[   48.529521] scull_test: Parameters:
[   48.529665] scull_test: Resquest(0) or Response(1): 0
[   48.529890] scull_test: Mode: 1
[   48.530962] scull_test: number3
[   48.531246] scull_test: number1
[   48.570017] scull_test: Hello from Client!
[   48.570365] scull_test: -------------------------
[   48.570514] scull_test: Starting device!
[   48.570631] scull_test: waiting for changes.....
[   48.570791] scull_test: -------------------------
[   48.570947] scull_test: Rust scull module parameters sample (init)
[   48.571152] scull_test: Parameters:
[   48.571288] scull_test: Resquest(0) or Response(1): 0
[   48.571473] scull_test: Mode: 1
[   48.572241] scull_test: number3
modprobe: 'kernel/samples/rust/rust_scull.ko': Network is unreachable
~ # qemu-system-x86_64: Slirp: Failed to send packet, ret: -1

```

qemu-system-x86_64 -netdev user,id=mynet0,hostfwd=tcp::8080-:8080 -device e1000,netdev=mynet0

qemu-system-x86_64 -netdev user,id=mynet0,hostfwd=tcp::8080-:8080 -device e1000,netdev=mynet0 -hda <path_to_disk_image> -m 1024

