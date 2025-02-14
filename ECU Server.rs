
// SPDX-License-Identifier: GPL-2.0

//! Rust server sample.


use kernel::{
    kasync::{executor::{workqueue::Executor as WqExecutor, AutoStopHandle, Executor},
    net::{TcpListener, TcpStream}},
    net::{self, Ipv4Addr, SocketAddr, SocketAddrV4},
    prelude::*, spawn_task,
    sync::{Arc, ArcBorrow} 
};

use kernel::net::*;

async fn echo_server(stream: TcpStream) -> Result {
    let mut buf = [0u8; 8];
    
        let n = stream.read(&mut buf).await?;
        pr_info!("ECU server Read") ;
        pr_info!("------------------------------------") ; 
        if n == 0 {
            pr_info!("Not getting anything!");
            return Ok(());
        }
        
    Ok(())
}

async fn accept_loop(listener: TcpListener, executor: Arc<impl Executor>) {
    loop {
        if let Ok(stream) = listener.accept().await {
            pr_info!("Client is connected!");
            let _ = spawn_task!(executor.as_arc_borrow(), echo_server(stream));
        }
    }
}

fn start_listener(ex: ArcBorrow<'_, impl Executor + Send + Sync + 'static>) -> Result {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::ANY, 8080));
    let listener = TcpListener::try_new(net::init_ns(), &addr)?;
    pr_info!("The start_listener function has been invoked ...") ; 
    spawn_task!(ex, accept_loop(listener, ex.into()))?;
    Ok(())
}

struct RustEchoServer {
    _handle: AutoStopHandle<dyn Executor>,
}

impl kernel::Module for RustEchoServer {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {

        pr_info!("Hello from Server!\n");
        pr_info!("-------------------------\n");
        pr_info!("Starting!\n");
        pr_info!("-------------------------\n");
        
        let handle = WqExecutor::try_new(kernel::workqueue::system())?;
        start_listener(handle.executor())?;
        pr_info!("Listened to Client"); 
        Ok(Self {
            _handle: handle.into(),
        })
    }
}

module! {
    type: RustEchoServer,
    name: "rust_ecu_server",
    description: "Rust tcp sample",
    license: "GPL v2",
}
