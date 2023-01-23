use {
    tonic::{Request, Response, Status, codegen::futures_core::Stream},
    std::{
        pin::Pin,
        time::Duration,
    },
    tokio_stream,
    tokio::sync::mpsc,
};



use tokio::runtime::Builder;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    pipe::{
        broadcast_server::{
            Broadcast,
        },
        Subscription,
        Update, self,
    }, 
    server::FirehosePipe, 
    basic,
};

type SubscribeResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Update, Status>> + Send>>;

#[tonic::async_trait]
impl Broadcast for FirehosePipe{
    type SubscribeStream = ResponseStream;

    async fn run(&self, _req: Request<basic::Empty>)->Result<Response<basic::Empty>,Status>{
    
        return Ok(Response::new(basic::Empty{}))
    }
    async fn subscribe(&self, req: Request<Subscription>)->SubscribeResult<Self::SubscribeStream>{
        println!("EchoServer::server_streaming_echo");
        println!("\tclient connected from: {:?}", req.remote_addr());
        let (tx, rx) = mpsc::channel(128);
        //let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));
        let mut i_rx = self.mgr.subscribe();
        let manager = self.mgr.clone();
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.spawn(async move {
            let mut r_i;
            loop{
                r_i =i_rx.recv().await;
                if r_i.is_err(){
                    if tx.send(Err(Status::from_error(Box::new(r_i.err().unwrap())))).await.is_err(){
                        break
                    }
                    return
                }
                let r = manager.account_read(r_i.unwrap()).await;
                if r.is_err(){
                    if tx.send(Err(Status::from_error(Box::new(r.err().unwrap())))).await.is_err(){
                        break
                    }
                    return
                }
                let a = r.unwrap();
                if tx.send(Ok(pipe::Update{
                    event: Some(pipe::update::Event::Account(a)),
                }),).await.is_err(){
                    break
                }

            }
        });
        let output_stream = ReceiverStream::new(rx);
        
        Ok(Response::new(
            Box::pin(output_stream) as Self::SubscribeStream
        ))
    }

}