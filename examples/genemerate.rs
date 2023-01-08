use hexafreeze::*;
use std::time::Duration;

const GENERATE_NUMBER: i64 = 4096000;

#[tokio::main]
async fn main() {
    let g = Generator::new(0, *DEFAULT_EPOCH).unwrap();
    for x in 0..GENERATE_NUMBER {
        if x % 10000 == 0 {
            println!("{}", x);
        }
        /* 
        if let Ok(x) = tokio::time::timeout(Duration::from_millis(5), g.generate()).await {
            x.unwrap();
        } else {
            println!("{:#?}", g.check_internal_generator());
        }
        */
        g.generate().await.expect("Bad");
        if let Err(x) = g.check_internal_generator() {
            println!("{}", x);
        }
    }
}