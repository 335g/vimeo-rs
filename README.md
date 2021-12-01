# vimeo-rs

```rust
use vimeo_rs as vimeo;

#[tokio::main]
async fn main() {
    let at: Url = ...
    let from: Url = ...
    let save_path: PathBuf = ...
    let user_agent: &'static str = ...

    match vimeo::get_movie(at, from, save_path, USER_AGENT, None).await {
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
```