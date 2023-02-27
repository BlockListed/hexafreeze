# Hexafreeze
A library to asynchronously generate Snowflake IDs.

# What is a snowflake
Snowflakes were developed by twitter for creating time sortable ids, which are able to be quickly generated without syncronisation even in distributed compute clusters.

Snowflakes have the following layout:

![Snowflake ID layout](https://upload.wikimedia.org/wikipedia/commons/5/5a/Snowflake-identifier.png)

# Usage
First you need to include dependencies. These are the recommended features. Tokio may be slimmed down by enabling individual features instead of `full`.
```ignore
[dependencies]
hexafreeze = "0.5"
tokio = {version = "1", features = ["full"]}
```

[`Generator`] is the interface for the generation of snowflakes.
Snowflakes require an `epoch`, basically the start time of the Snowflake, it needs to be in the past and be less than ~ 69 years ago. [`DEFAULT_EPOCH`] should be fine for most applications until 2079.
It is thread-safe, therefore you do not need a Mutex to contain it.
It is recommend to use the same generator in all places in a rust application, something like `once_cell` may be useful for this.
```rust
use hexafreeze::Generator;
use hexafreeze::DEFAULT_EPOCH;

#[tokio::main]
async fn main() {
    // If your system is not distributed using `0` as the `node_id` is perfectly fine.
    // The `DEFAULT_EPOCH` always needs to be dereferenced.
    let gen = Generator::new(0, *DEFAULT_EPOCH).unwrap();

    // The `generate` function is async and non-blocking.
    let id: i64 = gen.generate().await.unwrap();
}
```

# Details
* Unlike Twitter's reference implementation, the sequence does **not** get reset every millisecond.