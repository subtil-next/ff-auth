# FF Auth Examples

This directory contains examples demonstrating how to use the ff-auth library.

## Global Example

The `global_example.rs` demonstrates how to use the Global client with proper logging and command-line argument parsing.

### Usage

```bash
cargo run --example global_example --features global,examples -- \
    --username "your_username" \
    --password "your_password" \
    --log-level debug
```

### Available Options

- `--username` / `-u`: Your FF account username (required)
- `--password` / `-p`: Your FF account password (required)  
- `--otp` / `-o`: One-time password for 2FA (optional)
- `--region` / `-r`: Login region code (optional)
- `--free-trial`: Use free trial login (optional flag)
- `--log-level`: Set logging level (trace, debug, info, warn, error) - defaults to "info"

### Example with all options

```bash
cargo run --example global_example --features global,examples -- \
    --username "player@example.com" \
    --password "mypassword" \
    --otp "123456" \
    --region 1 \
    --free-trial \
    --log-level trace
```

This example shows:
- How to use CLAP for command-line argument parsing
- How to set up tracing-subscriber for detailed logging
- How to create and configure a LoginRequest 
- How to use the GlobalClient to authenticate
- Proper error handling and logging