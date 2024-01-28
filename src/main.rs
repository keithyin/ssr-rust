mod socks5;
mod proxy;
mod remote;
mod proxy_tokio;
mod remote_tokio;

use tokio;
use std::env;


/// https://jiajunhuang.com/articles/2019_06_06-socks5.md.html.
/// 文中的 client 可以理解就是 浏览器
/// 文中的 server 对于ssr，包括两部分，一个是 本地的 server + 远程的 server。
pub fn main() {
    let args = env::args().collect::<Vec<String>>();
    let tag = args[1].clone();
    match tag.as_str() {
        "proxy" => proxy::proxy_server_v3(),
        "server" => remote::remote_server(),
        "proxy_tokio" => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(proxy_tokio::proxy_server_v3()).expect("proxy tokio err");
        },
        "server_tokio" => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(remote_tokio::remote_server()).expect("server tokio err");
        }
        _ => panic!("invalid args {}", tag.as_str()),
    }
}