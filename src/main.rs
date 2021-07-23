use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::stream::StreamExt;

async fn read_from_local() -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8089").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            let mut buf = [0; 1024];

            loop {
                let n = reader.read(&mut buf).await.unwrap();
                println!("bytes:{}, data:{}", n, String::from_utf8_lossy(&buf[0..n]));
                if n ==0 || buf[n-1] == '\0' as u8{
                    break;
                }
            }
            writer.write("hello client".as_bytes()).await;
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

#[tokio::main]
pub async fn main() -> io::Result<()> {
    send_msg_to_remote().await?;

    Ok(())
}