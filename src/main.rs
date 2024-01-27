mod socks5;
mod proxy;
mod remote;

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::stream::StreamExt;
use std::env;

async fn read_from_local() -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:1080").await?;
    loop {
        let (mut proxy, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            let mut n = proxy.read(&mut buf).await.unwrap();
            // println!("first read:{:?}", &buf[0..n]);
            proxy.write(&[5, 0]).await.unwrap();
            n = proxy.read(&mut buf).await.unwrap();
            // println!("second read:{:?}", &buf[0..n]);
            let socks5_data = socks5::Socks5::new(&buf[0..n]);
            // println!("{:?}", socks5_data);

            // connect to ther server
            let mut remote = TcpStream::connect("127.0.0.1:8086").await.unwrap();
            remote.write(&socks5_data.encrypt()).await.unwrap();


            // get the remote response to the local
            let reply =  [5, 0, 0, 1, 0, 0, 0, 0, 8, 174];
            proxy.write(&reply).await.unwrap();
           tokio::select! {
               _ = proxy.readable() => {
                   let mut buf = [0; 4096];
                   let mut n = proxy.read(&mut buf).await.unwrap();
                   let write_n = remote.write(&buf[0..n]).await.unwrap();
                   assert_eq!(write_n, n);
                   println!("proxy read:{}, remote write:{}, {:?}", n, write_n, socks5_data.get_addr());

               }
               _ = remote.readable() => {
                   let mut buf = [0; 4096];
                   let mut n = remote.read(&mut buf).await.unwrap();
                   let write_n = proxy.write(&buf[0..n]).await.unwrap();
                   assert_eq!(write_n, n);
                   println!("remote read:{}, proxy write:{}, {:?}", n, write_n, socks5_data.get_addr());
               }
           }

        });
    }
    Ok(())
}

async fn send_msg_to_remote() -> io::Result<()> {
    let mut stream = TcpStream::connect("47.241.7.128:80").await?;
    println!("connected");
    stream.write("hello boy".as_bytes()).await;
    let mut buf = [0 as u8; 1024];
    let n = stream.read(&mut buf).await?;
    println!("{:?}", String::from_utf8_lossy(&buf[0..n]));
    Ok(())
}

fn read_from_local_v2() {

}



// #[tokio::main]
// pub async fn main() -> io::Result<()> {
//    read_from_local().await?;
//
//     Ok(())
// }

/// https://jiajunhuang.com/articles/2019_06_06-socks5.md.html.
/// 文中的 client 可以理解就是 浏览器
/// 文中的 server 对于ssr，包括两部分，一个是 本地的 server + 远程的 server。
pub fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mode = args[1].clone();

    match mode.as_str() {
        "proxy" => proxy::proxy_server_v3(),
        "server" => remote::remote_server(),
        _ => panic!("invalid args {}", mode.as_str()),

    }
}