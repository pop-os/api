# Pop!_OS API

Rust crate for interacting with the Pop!_OS API

```rust
#[macro_use]
extern crate fomat_macros;

use futures::executor;
use pop_os_api::builds::Build;

fn main() {
    executor::block_on(async move {
        let build = Build::get("20.04", "intel").await?;

        pint!(
            "build:   " (build.build) "\n"
            "sha_sum: " (build.sha_sum) "\n"
            "size:    " (build.size) "\n"
            "url:     " (build.url) "\n"
        );
    })
}
```
