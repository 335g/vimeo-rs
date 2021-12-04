# vimeo-rs

### How to use

```
[dependencies]
vimeo-rs = { version = "0.1", features = ["progressbar"] }
```

### For example

```rust
pub const USER_AGENT: &'static str = "...";
use vimeo_rs as vimeo;
use vimeo::{ProgressBar, ProgressStyle, MultiProgress};

#[tokio::main]
async fn main() {
    let mut handles = vec![];
    let bars = MultiProgress::new();
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}) {msg}")
        .progress_chars("#>-");

    // 1st content
    let pb1 = bars.add(ProgressBar::new(1));
    pb1.set_style(style.clone());
    let handle = tokio::spawn(async move {
        let at = "...";
        let from = "...";
        let downloading_msg = "downloading 1".to_string();
        let finished_msg = "finished 1".to_string();

        vimeo::get_movie(at, from, "a1.mp4", USER_AGENT, pb1, Some(downloading_msg), Some(finished_msg)).await
    });
    handles.push(handle);
    
    // 2nd content
    let pb2 = bars.add(ProgressBar::new(1));
    pb2.set_style(style.clone());
    let handle = tokio::spawn(async move {
        let at = "...";
        let from = "...";
        let downloading_msg = "downloading 2".to_string();
        let finished_msg = "finished 2".to_string();

        vimeo::get_movie(at, from, "a2.mp4", USER_AGENT, pb2, Some(downloading_msg), Some(finished_msg)).await
    });
    handles.push(handle);

    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```