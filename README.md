# Raycast AI Proxy

This is a simple [Raycast AI](https://raycast.com/) API proxy by Rust. It allows you to use the [Raycast AI](https://raycast.com/ai) app without a subscription.
It's a simple proxy that forwards requests from Raycast, converts the format, and returns the response in real-time.

inspired by [Raycast AI Proxy](https://github.com/yufeikang/raycast_api_proxy)

## Usage

1. Clone this repository
2. Use `./scripts/cert_gen.py --domain backend.raycast.com  --out ./cert` to generate a self-signed certificate
3. You can modify the parameters of `ChatMLClient` according to your needs
```rust
#[derive(Debug, TypedBuilder, Clone)]
pub struct ChatMLClient {
    #[builder(default = "http://localhost:8080/v1/chat/completions".to_string())]
    server_url: String,
    #[builder(default = "no-key".to_string())]
    secret_key: String,
    #[builder(default = "gpt-3.5-turbo".to_string())]
    model: String,
}
``` 
simply, you can use `llama.cpp` server     
4. `cargo build -r` && `sudo ./target/release/raycast-api-proxy`

### Configuration

1. Modify `/etc/host` to add the following line:

```
127.0.0.1 backend.raycast.com
::1 backend.raycast.com
```

The purpose of this modification is to point `backend.raycast.com` to the localhost instead of the actual `backend.raycast.com`. You can also add this
record in your DNS server.

2. Add the certificate trust to the system keychain

Open the CA certificate in the `cert` folder and add it to the system keychain and **TRUST IT**.
This is **necessary** because the Raycast AI Proxy uses a self-signed certificate and it must be trusted to work properly.