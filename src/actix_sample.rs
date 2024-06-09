use actix::prelude::*;


#[derive(Message)]
#[rtype(result = "()")]
struct Measurement(i64);

#[derive(Message)]
#[rtype(result = "()")]
struct ResultSum(i64);

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

impl Handler<ResultSum> for SumPrinter {
    type Result = ();

    fn handle(&mut self, msg: ResultSum, ctx: &mut Self::Context) -> Self::Result {
        println!("Result: {}", msg.0);
    }
}

impl Handler<Measurement> for SumActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: Measurement, _ctx: &mut Context<Self>) -> Self::Result {
        self.sum = self.sum + msg.0;

        let sum = self.sum;
        let dst = self.dst.clone();
        Box::pin(async move {
            let _ = dst.send(ResultSum(sum)).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use actix::{Actor, Arbiter, System};
    use crate::actix_sample::{Measurement, SumActor, SumPrinter};

    #[actix_rt::test]
    async fn test_actix() -> Result<(), anyhow::Error> {
        let printActor = SumPrinter {}.start();
        let addr = SumActor { sum: 0, dst: printActor }.start();

        for i in 0..10 {
            let _ = addr.send(Measurement(i)).await?;
        }

        Ok(())
    }
}