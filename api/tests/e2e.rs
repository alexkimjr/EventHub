use std::process::Command;
use std::time::Duration;
use std::thread::sleep;

#[test]
fn e2e_post_and_consume() {
    // Run this end-to-end test only in GitHub Actions CI.
    if std::env::var("GITHUB_ACTIONS").is_err() {
        eprintln!("skipping e2e: GitHub Actions only");
        return;
    }

    // post to API
    let resp = ureq::post("http://127.0.0.1:8080/ingest")
        .set("Content-Type", "application/json")
        .send_string(r#"{"e2e":"hello"}"#);

    // `send_string` returns a Result; handle error cases before checking status
    let resp = match resp {
        Ok(r) => r,
        Err(e) => panic!("POST to API failed: {:?}", e),
    };

    let status = resp.status();
    assert!(status >= 200 && status < 300, "POST to API failed: {}", status);

    // allow Kafka to receive
    sleep(Duration::from_secs(5));

    // consume one message from topic first_topics
  
        // use kafkacat to consume a single message from the topic in CI
        let output = Command::new("kcat")
            .arg("-C")
            .arg("-b")
            .arg("127.0.0.1:9092")
            .arg("-t")
            .arg("first_topics")
            .arg("-c")
            .arg("1")
            .arg("-o")
            .arg("-1")
            .arg("-T")
            .output()
            .expect("failed to run kcat");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"), "consumer did not receive message: {}", stdout);
}
