use std::io::Read;
use reqwest::Client;
use scraper::{Html, Selector};
use vimeo_rs::{PlayerConfig, Content, player_config};



#[tokio::test]
async fn test() {
    let mut f = std::fs::File::open("data/b.html").expect("is file");
    let mut html = String::new();
    f.read_to_string(&mut html).unwrap();

    let obj = player_config(&html).await.expect("is player config");
    if let Some(cdn) = obj.dash_default_cdn() {
        let client = Client::builder().build().unwrap();
        let request = client.get(cdn.avc_url.clone()).build().unwrap();
        let content = client.execute(request)
            .await
            .expect("aaa")
            .json::<Content>()
            .await
            .expect("bbb");

        // for audio in content.audio_exps() {
        //     println!("{:?}", audio);
        // }

        // for video in content.video_exps() {
        //     println!("{:?}", video);
        // }

        // let autio1 = content.au

    }
}