use std;
use std::net::SocketAddr;
use std::io::{Read, Write};
use crate::socks5::Socks5;
use std::{thread, io};
use std::time::Duration;

/// 接收 proxy 的请求，执行指针的 http 请求
pub fn remote_server() {
    let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
    let listener = std::net::TcpListener::bind(addr).unwrap();
    loop {
        let (mut proxy_remote, _) = listener.accept().unwrap();
        let mut buf = [0;40960];
        let n = proxy_remote.read(&mut buf).unwrap();

        let sock5 = Socks5::decrypt(&buf[0..n]);

        println!("connected to {}:{}", sock5.get_addr(), sock5.get_port());

        let _ = thread::spawn(move || -> io::Result<()>{
            let mut dest = std::net::TcpStream::connect(format!("{}:{}", sock5.get_addr(), sock5.get_port())).unwrap();

            let mut buf = [0; 4096];
            loop {
                dest.set_nonblocking(false).unwrap();
                proxy_remote.set_nonblocking(true).unwrap();
                match proxy_remote.read(&mut buf) {
                    Ok(n) => {
                        dest.write_all(&buf[0..n])?;
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {

                    },
                    _ => {
                        println!("realease thread");
                        break;
                    }
                }
                proxy_remote.set_nonblocking(false)?;
                dest.set_nonblocking(true).unwrap();
                match dest.read(&mut buf) {
                    Ok(n) => {
                        proxy_remote.write_all(&buf[0..n]).unwrap();
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {

                    },
                    _ => {
                        println!("realease thread");
                        break;
                    }
                }
            }
            Ok(())
        });
    }
}