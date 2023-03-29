use gag::BufferRedirect;
use serde_json::{de, value};
use std::{collections::BTreeMap, io::stdout, io::Read};
use structured_logger::{json::new_writer, unix_ms, Builder};
use tokio::task::JoinSet;

#[test]
fn it_works() {
    let _ = Builder::new()
        .with_target_writer("buffer", new_writer(stdout()))
        .try_init();

    {
        let buf = BufferRedirect::stderr().unwrap();
        log::info!("hello world");
        let mut msg: String = String::new();
        buf.into_inner().read_to_string(&mut msg).unwrap();
        let msg = msg.as_str();
        // println!("JSON: {}", msg);
        assert_eq!('\n', msg.chars().last().unwrap());

        let res = de::from_str::<BTreeMap<String, value::Value>>(msg);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!("INFO", res.get("level").unwrap());
        assert_eq!("hello world", res.get("message").unwrap());
        assert_eq!("test", res.get("target").unwrap());
        assert!(unix_ms() - 999 <= res.get("timestamp").unwrap().as_u64().unwrap());
    }

    {
        let buf = BufferRedirect::stdout().unwrap();
        let mut kv = BTreeMap::<&str, &str>::new();
        kv.insert("uid", "user123");
        kv.insert("action", "upate_book");

        log::info!(target: "buffer",
            method = "GET",
            path = "/hello",
            status = 200_u16,
            kv = log::as_serde!(kv);
            "",
        );

        let mut msg: String = String::new();
        buf.into_inner().read_to_string(&mut msg).unwrap();
        let msg = msg.as_str();
        // println!("JSON: {}", msg);
        assert_eq!('\n', msg.chars().last().unwrap());

        let res = de::from_str::<BTreeMap<String, value::Value>>(msg);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!("INFO", res.get("level").unwrap());
        assert_eq!("", res.get("message").unwrap());
        assert_eq!("buffer", res.get("target").unwrap());
        assert_eq!("GET", res.get("method").unwrap());
        assert_eq!("/hello", res.get("path").unwrap());
        assert_eq!(200_u64, res.get("status").unwrap().as_u64().unwrap());
        assert!(unix_ms() - 999 <= res.get("timestamp").unwrap().as_u64().unwrap());

        let res_kv = res.get("kv").unwrap().as_object().unwrap();
        assert_eq!("user123", res_kv.get("uid").unwrap());
        assert_eq!("upate_book", res_kv.get("action").unwrap());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn multiple_threads_works() {
    let _ = Builder::new().try_init();

    let mut set = JoinSet::new();

    let mut seen = [false; 1000];
    for (i, _) in seen.iter().enumerate() {
        set.spawn(async move {
            log::info!("hello {}", i);
            i
        });
    }

    while let Some(res) = set.join_next().await {
        let idx = res.unwrap();
        seen[idx] = true;
    }

    for v in &seen {
        assert!(*v);
    }
}
