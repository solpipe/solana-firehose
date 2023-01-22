use std::pin::Pin;

use tonic::{Request, Response, Status, codegen::futures_core::Stream};

use crate::{
    pipe::{
        broadcast_server::{
            Broadcast,
        },
        Subscription,
        Update,
    }, 
    server::FirehosePipe, 
    basic,
};

type SubscribeResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Update, Status>> + Send>>;

#[tonic::async_trait]
impl Broadcast for FirehosePipe{
    type SubscribeStream = ResponseStream;

    async fn run(&self, req: Request<basic::Empty>)->Result<Response<basic::Empty>,Status>{
    
        return Ok(Response::new(basic::Empty{}))
    }
    async fn subscribe(&self, req: Request<Subscription>)->SubscribeResult<Self::SubscribeStream>{
        println!("EchoServer::server_streaming_echo");
        println!("\tclient connected from: {:?}", req.remote_addr());
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));
        Ok(())
    }

}