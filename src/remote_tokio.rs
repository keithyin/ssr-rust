use std;
use std::net::SocketAddr;
use crate::socks5::Socks5;
use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 接收 proxy 的请求，执行指针的 http 请求
pub async fn remote_server() -> io::Result<()>{
    let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut proxy_remote, _) = listener.accept().await?;
        let mut buf = [0;40960];
        let n = proxy_remote.read(&mut buf).await?;

        let sock5 = Socks5::decrypt(&buf[0..n]);
        println!("connected to {}:{}", sock5.get_addr(), sock5.get_port());

        tokio::spawn(async move {
            let talk_addr = format!("connected to {}:{}", sock5.get_addr(), sock5.get_port());
            let mut dest = TcpStream::connect(format!("{}:{}", sock5.get_addr(), sock5.get_port())).await?;
            let mut dest_buffer = [0_u8; 4096];
            let mut proxy_buffer = [0_u8; 4096];
            loop {
                tokio::select! {
                    res = proxy_remote.read(&mut proxy_buffer) => {
                        match res {
                            Ok(0) => break,
                            Ok(n) => {
                                dest.write_all(&proxy_buffer[0..n]).await?;
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {
                                panic!("this should not happen")
        
                            },
                            _ => {
                                println!("proxy read err: disconnected: {talk_addr}");
                                break;
                            }
                        }
                        
                    },

                    res = dest.read(&mut dest_buffer) => {
                        match res {
                            Ok(0) => break,
                            Ok(n) => {
                                proxy_remote.write_all(&dest_buffer[0..n]).await?;
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {
                                panic!("this should not happen")
        
                            },
                            _ => {
                                println!("dest readd err: disconnected: {talk_addr}");

                                break;
                            }
                        }
                    }
                }
            }
            Ok::<(), io::Error>(())
        });
    }
}