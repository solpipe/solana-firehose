use tonic::{Request, Response, Status};

use crate::{pipe::broadcast_server::Broadcast, server::FirehosePipe, basic};



#[tonic::async_trait]
impl Broadcast for FirehosePipe{
    async fn run<'a>(&'a self, msg: Request<basic::Empty>)->Result<Response<basic::Empty>,Status>{

        return Ok(Response::new(basic::Empty{}))
    }

}