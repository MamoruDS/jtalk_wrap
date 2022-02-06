# jtalk_warp

## Example

```rust
let cookies = String::from("j_talk_session=_; XSRF-TOKEN=_;");

let mut jtalk_client = JTalk::new()
        .set_account("foo@bar.com", "password")
        .set_remember(true)
        .load_cookies(&cookies);
// jtalk_client.login().await;

let (id, result) = jtalk_client.convert(&text).await;
// access your convert result from https://j-talk.com/{id}
println!("{}", json!(result));
```
