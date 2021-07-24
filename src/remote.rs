use std;
use std::net::SocketAddr;
use std::io::{Read, Write};
use crate::socks5::Socks5;
use std::{thread, io};
use std::time::Duration;

pub fn remote_server() {
    let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
    let listener = std::net::TcpListener::bind(addr).unwrap();
    loop {
        let (mut proxy_remote, _) = listener.accept().unwrap();
        let mut buf = [0;4096];
        let mut n = proxy_remote.read(&mut buf).unwrap();

        let sock5 = Socks5::decrypt(&buf[0..n]);

        println!("{}:{}", sock5.get_addr(), sock5.get_port());
        // let dest_addr:SocketAddr = format!("{}:{}", sock5.get_addr(), sock5.get_port()).parse().unwrap();
        let mut dest = std::net::TcpStream::connect(format!("{}:{}", sock5.get_addr(), sock5.get_port())).unwrap();

        let t = thread::spawn(move || -> io::Result<()>{
            let mut buf = [0; 4096];
            loop {
                dest.set_nonblocking(false).unwrap();
                proxy_remote.set_nonblocking(true).unwrap();
                match proxy_remote.read(&mut buf) {
                    Ok(n) => {
                        dest.write_all(&buf[0..n])?;
                        println!("SERVER: send data to dest");
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {

                    },
                    _ => {
                        break;
                    }
                }
                proxy_remote.set_nonblocking(false)?;
                dest.set_nonblocking(true).unwrap();
                match dest.read(&mut buf) {
                    Ok(n) => {
                        proxy_remote.write_all(&buf[0..n]).unwrap();
                        println!("SERVER: send data to proxy");
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {

                    },
                    _ => {
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        });
    }
}