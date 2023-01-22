use tonic::{Request, Response, Status};

use crate::{pipe::utility_server::Utility, server::FirehosePipe, basic};



#[tonic::async_trait]
impl Utility for FirehosePipe{
    async fn run<'a>(&'a self, msg: Request<basic::Empty>)->Result<Response<basic::Empty>,Status>{

        return Ok(Response::new(basic::Empty{}))
    }
}