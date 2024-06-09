use actix::prelude::*;


#[derive(Message)]
#[rtype(result = "i64")]
struct Measurement(i64);

struct SumActor {
    sum: i64,
}

impl Actor for SumActor {
    type Context = Context<Self>;
}

impl Handler<Measurement> for SumActor {
    type Result = i64;

    fn handle(&mut self, msg: Measurement, _ctx: &mut Context<Self>) -> Self::Result {
        self.sum = self.sum + msg.0;
        self.sum
    }
}

#[cfg(test)]
mod tests {
    use actix::{Actor, Arbiter, System};
    use crate::actix_sample::{Measurement, SumActor};

    #[actix_rt::test]
    async fn test_actix() -> Result<(), anyhow::Error> {
        let addr = SumActor { sum: 0 }.start();

        for i in 0..10 {
            let res = addr.send(Measurement(i)).await;
            println!("RESULT: {}", res.unwrap());
        }

        Ok(())
    }
}