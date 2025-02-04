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
