use std;
use crate::socks5;

use std::io;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// this works.
pub async fn proxy_server_v3() -> io::Result<()>{
    let addr: SocketAddr  = "127.0.0.1:1080".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut proxy, _) = listener.accept().await?;
        let mut buf = [0; 40960];

        let mut n = proxy.read(&mut buf).await?;
        
        proxy.write_all(&[5, 0]).await?;
        n = proxy.read(&mut buf).await?;
        if n < 4 {
            continue;
        }
        let socks5_data = socks5::Socks5::new(&buf[0..n]);
        
        println!("connect to {}:{}", socks5_data.get_addr(), socks5_data.get_port());

        tokio::spawn(async move {
            let talk_addr = socks5_data.get_addr().to_string();
            let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();

            let mut remote = TcpStream::connect(addr).await?;
            remote.write_all(&socks5_data.encrypt()).await?;

            let reply =  [5, 0, 0, 1, 0, 0, 0, 0, 8, 174];
            proxy.write_all(&reply).await?;
            
            let mut proxy_buf = [0_u8; 4096];
            let mut remote_buf = [0_u8; 4086];

            loop {
                tokio::select! {
                    res = proxy.read(&mut proxy_buf) => {
                        match res {
                            Ok(n) => {
                                if n > 0 {
                                    remote.write_all(&proxy_buf[0..n]).await?;
                                }
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

                    res = remote.read(&mut remote_buf) => {
                        match res {
                            Ok(n) => {
                                if n > 0 {
                                    proxy.write_all(&remote_buf[0..n]).await?;
                                }

                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {
                                panic!("this should not happen")
    
                            },
                            _ => {
                                println!("remote read err: disconnected: {talk_addr}");

                                break;
                            }
                        }
                    }
                };
            }
            Ok::<(), io::Error> (())
        });
    }
}