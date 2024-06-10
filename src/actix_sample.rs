use std::time::Duration;
use actix::prelude::*;




















#[derive(Message)]
#[rtype(result = "()")]
struct Measurement(i64);

#[derive(Message)]
#[rtype(result = "()")]
struct ResultSum(i64);

struct Emitter {
    dst: Addr<SumActor>,
}

struct SumActor {
    sum: i64,
    dst: Addr<SumPrinter>,
}

struct SumPrinter;


impl Actor for SumPrinter {
    type Context = Context<Self>;
}

impl Actor for SumActor {
    type Context = Context<Self>;
}

impl Actor for Emitter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(100), |act, ctx| {
            act.dst.do_send(Measurement(1));
        });
    }
}

impl Handler<ResultSum> for SumPrinter {
    type Result = ();

    fn handle(&mut self, msg: ResultSum, _ctx: &mut Self::Context) -> Self::Result {
        println!("Got: {}", msg.0);
    }
}

impl Handler<Measurement> for SumActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: Measurement, _ctx: &mut Context<Self>) -> Self::Result {
        self.sum += msg.0;

        let sum = self.sum;
        let dst = self.dst.clone();
        Box::pin(async move {
            let _ = dst.send(ResultSum(sum)).await;
        })
    }
}


#[derive(Message)]
#[rtype(result = "Result<String, ()>")]
enum MyMessage {
    Ping,
    Pong,
    Shutdown,
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use actix::{Actor};
    use crate::actix_sample::{Emitter, SumActor, SumPrinter};

    #[actix_rt::test]
    async fn test_actix() -> Result<(), anyhow::Error> {
        let printer = SumPrinter {}.start();
        let sum_actor = SumActor { sum: 0, dst: printer }.start();
        let emitter = Emitter { dst: sum_actor }.start();

        tokio::time::sleep(Duration::from_secs(3)).await;

        Ok(())
    }
}