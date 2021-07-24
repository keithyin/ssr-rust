
#[derive(Debug)]
pub struct Socks5 {
    mode: u8,
    addr_type: u8,
    addr: String,
    port: u32,
    port_high8: u8,
    port_low8: u8,
    addr_len: u8,
}

impl Socks5 {
    pub fn new(data: &[u8]) ->Self {
        let mode = data[1];
        let addr_type = data[3];
        let mut addr_len = 0;
        let mut addr = "".to_string();
        let mut port: u32 = 0;
        let mut port_high8 = 0;
        let mut port_low8 = 0;
        if addr_type == 3 {
            addr_len = data[4];
            addr = String::from_utf8_lossy(&data[5..((5+addr_len) as usize)]).to_ascii_lowercase();
            port = ((data[(5+addr_len as usize)] as u32) << 8) + (data[(5+addr_len + 1) as usize] as u32);
            port_high8 = data[(5+addr_len as usize)];
            port_low8 = data[(5+addr_len + 1) as usize];
        }
        Socks5{
            mode,
            addr_type,
            addr,
            port,
            port_high8,
            port_low8,
            addr_len,
        }
    }

    pub fn encrypt(&self) -> Vec<u8> {
        let mut res = vec![];
        res.push(self.addr_type);
        res.push(self.addr_len);
        res.extend_from_slice(self.addr.as_bytes());
        res.push(self.port_high8);
        res.push(self.port_low8);
        res
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }
}