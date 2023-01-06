# Hexafreeze
A library to asynchronously generate Snowflake IDs.

# What is a snowflake
Snowflakes where developed by twitter for creating time sortable ids, which where able to be quickly generated without syncronisation even in distributed compute clusters.

Snowflakes have the following layout:

![Snowflake ID layout](https://upload.wikimedia.org/wikipedia/commons/5/5a/Snowflake-identifier.png)

# Usage
[`Generator`] is the interface for the generation of snowflakes.
It is thread-safe, therefore you do not need a Mutex to contain it.
It is recommend to use the same generator for all purposes in a rust application. Something like `once_cell` may be useful for this.
```rust
use hexafreeze::Generator;
use chrono::prelude::*;

let epoch = DateTime::from_utc(NaiveDate::from_ymd(2022, 1, 1).and_hms(0, 0, 0), Utc)

// If your system is not distributed using `0` as the `node_id` is perfectly fine.
let gen = Generator::new(0, epoch)

// The `generate` function is async and non-blocking.
let id: i64 = gen.generate().await
```

# Details
* The generation happens on a seperate thread to avoid error-prone synchronisation issues.
* Unlike Twitter's reference implementation the sequence does **not** get reset every millisecond.