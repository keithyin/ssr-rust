use std;
use std::io::{Read, Write};
use crate::socks5;
use mio::net::TcpStream;
use mio::net::TcpListener;
use std::sync::{Mutex, Arc};
use std::{thread, io};
use std::net::SocketAddr;
use mio;
use mio::event::Event;
use mio::{Token, Interest, Poll};
use futures::future::MapInto;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;
use std::time::Duration;

struct Scheduler {
    // (proxy, remote)
    pairs: Arc<Mutex<Vec<(TcpStream, TcpStream)>>>,
}

impl Scheduler {

    pub fn new() -> Self {
        Scheduler{
            pairs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn append(&mut self, proxy: TcpStream, remote: TcpStream) {
        self.pairs.as_ref().lock().unwrap().push((proxy, remote));
    }

    pub fn schedule(&mut self) {
        let pairs = Arc::clone(&self.pairs);

    }
}

pub fn wait_for_event_ready(poll: &mut mio::Poll, events: &mut mio::Events, token: &Token) {
   loop{
       println!("waiting for {:?}", token);
       poll.poll(events, None).unwrap();
       for e in events.iter() {
           if e.token() == *token {
               println!("waiting for {:?}. DONE", token);
               return;
           }
       }
   }

}

pub fn which_is_ready(proxy: &mut TcpStream, remote: &mut TcpStream) -> Token {
    Token(0)
}

pub fn next_token(token: &Token) -> Token {
    Token(token.0 + 1)
}

pub fn proxy_server() {
    let addr: SocketAddr  = "127.0.0.1:1080".parse().unwrap();
    let mut listener = TcpListener::bind(addr).unwrap();

    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(3);
    let listener_token = Token(0);
    poll.registry().register(&mut listener, listener_token, Interest::READABLE).unwrap();

    loop {

        wait_for_event_ready(&mut poll, &mut events, &listener_token);
        let (mut proxy, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];

        let proxy_token = next_token(&listener_token);
        poll.registry().register(&mut proxy, proxy_token, Interest::READABLE).unwrap();
        wait_for_event_ready(&mut poll, &mut events, &proxy_token);
        let mut n = proxy.read(&mut buf).unwrap();
        // println!("first read:{:?}", &buf[0..n]);

        // wait_for_stream_write(&mut proxy, "first write".to_string());
        poll.registry().reregister(&mut proxy, proxy_token, Interest::WRITABLE).unwrap();
        wait_for_event_ready(&mut poll, &mut events, &proxy_token);
        proxy.write(&[5, 0]).unwrap();

        poll.registry().reregister(&mut proxy, proxy_token, Interest::READABLE).unwrap();
        wait_for_event_ready(&mut poll, &mut events, &proxy_token);
        n = proxy.read(&mut buf).unwrap();
        // println!("second read:{:?}", &buf[0..n]);

        let socks5_data = socks5::Socks5::new(&buf[0..n]);
        // println!("{:?}", socks5_data);

        // connect to ther server
        let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
        let mut remote = TcpStream::connect(addr).unwrap();
        let remote_token = next_token(&proxy_token);
        poll.registry().register(&mut remote, remote_token, Interest::WRITABLE).unwrap();
        remote.write_all(&socks5_data.encrypt()).unwrap();


        // get the remote response to the local
        let reply =  [5, 0, 0, 1, 0, 0, 0, 0, 8, 174];
        poll.registry().reregister(&mut proxy, proxy_token, Interest::WRITABLE).unwrap();
        wait_for_event_ready(&mut poll, &mut events, &proxy_token);
        proxy.write_all(&reply).unwrap();

        thread::spawn(move || {
            let mut buf = [0; 4096];
            loop {
                let token = which_is_ready(&mut proxy, &mut remote);
                if token == Token(0) {
                    let n = proxy.read(&mut buf).unwrap();
                    let write_n = remote.write(&buf[0..n]).unwrap();
                }else {
                    let n = remote.read(&mut buf).unwrap();
                    let write_n = proxy.write(&buf[0..n]).unwrap();
                }
            }

        });

    }
}


pub fn proxy_server_v2() {
    let addr: SocketAddr  = "127.0.0.1:1080".parse().unwrap();
    let mut listener = TcpListener::bind(addr).unwrap();

    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);
    let listener_token = Token(0);
    poll.registry().register(&mut listener, listener_token, Interest::READABLE).unwrap();

    // let mut connections = HashMap::new();

    loop {
        poll.poll(&mut events, None).unwrap();
        for e in events.iter() {
            match e.token() {
                listener_token=> {
                    let mut events = mio::Events::with_capacity(1024);
                    let (mut proxy, _) = listener.accept().unwrap();
                    let mut buf = [0; 4096];
                    let proxy_token = next_token(&listener_token);
                    poll.registry().register(&mut proxy, proxy_token, Interest::READABLE).unwrap();
                    wait_for_event_ready(&mut poll, &mut events, &proxy_token);
                    let mut n = proxy.read(&mut buf).unwrap();
                    // println!("first read:{:?}", &buf[0..n]);

                    // wait_for_stream_write(&mut proxy, "first write".to_string());
                    poll.registry().reregister(&mut proxy, proxy_token, Interest::WRITABLE).unwrap();
                    wait_for_event_ready(&mut poll, &mut events, &proxy_token);
                    proxy.write(&[5, 0]).unwrap();

                    poll.registry().reregister(&mut proxy, proxy_token, Interest::READABLE).unwrap();
                    wait_for_event_ready(&mut poll, &mut events, &proxy_token);
                    n = proxy.read(&mut buf).unwrap();
                    // println!("second read:{:?}", &buf[0..n]);

                    let socks5_data = socks5::Socks5::new(&buf[0..n]);
                    // println!("{:?}", socks5_data);

                    // connect to ther server
                    let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
                    let mut remote = TcpStream::connect(addr).unwrap();
                    let remote_token = next_token(&proxy_token);
                    poll.registry().register(&mut remote, remote_token, Interest::WRITABLE).unwrap();
                    remote.write(&socks5_data.encrypt()).unwrap();


                    // get the remote response to the local
                    let reply =  [5, 0, 0, 1, 0, 0, 0, 0, 8, 174];
                    poll.registry().reregister(&mut proxy, proxy_token, Interest::WRITABLE).unwrap();
                    wait_for_event_ready(&mut poll, &mut events, &proxy_token);
                    proxy.write(&reply).unwrap();
                }
                token => {

                }
            }
        }
    }
}

/// this works.
pub fn proxy_server_v3() {
    let addr: SocketAddr  = "127.0.0.1:1080".parse().unwrap();
    let mut listener = std::net::TcpListener::bind(addr).unwrap();
    let mut thread_pool = vec![];
    loop {
        let (mut proxy, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];

        let mut n = proxy.read(&mut buf).unwrap();

        proxy.write_all(&[5, 0]).unwrap();
        n = proxy.read(&mut buf).unwrap();

        let socks5_data = socks5::Socks5::new(&buf[0..n]);
        // println!("{:?}", socks5_data);

        // connect to ther server
        let addr: SocketAddr  = "127.0.0.1:8086".parse().unwrap();
        let mut remote = std::net::TcpStream::connect(addr).unwrap();
        remote.write_all(&socks5_data.encrypt()).unwrap();


        // get the remote response to the local
        let reply =  [5, 0, 0, 1, 0, 0, 0, 0, 8, 174];
        proxy.write_all(&reply).unwrap();

        let t = thread::spawn(move || -> io::Result<()>{
            let mut buf = [0; 4096];
            loop {
                remote.set_nonblocking(false).unwrap();
                proxy.set_nonblocking(true).unwrap();
                match proxy.read(&mut buf) {
                    Ok(n) => {
                        remote.write_all(&buf[0..n])?;
                        println!("PROXY: send data to remote");
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock  => {

                    },
                    _ => {
                        break;
                    }
                }
                proxy.set_nonblocking(false)?;
                remote.set_nonblocking(true).unwrap();
                match remote.read(&mut buf) {
                    Ok(n) => {
                        proxy.write_all(&buf[0..n]).unwrap();
                        println!("PROXY: send data to local");
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

        thread_pool.push(t);
        println!("num threads: {}", thread_pool.len());
    }
}