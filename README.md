# Shellcove

## Overview

Shellcove is a Rust-based microservice designed to function as an SSH gateway between broswer base 
Terminal emulators and ssh servers. It enables shell terminals to communicate with the service 
using WebSockets. The service then forwards the requests to an SSH server using the SSH protocol.

## Road Map

- [ ] Record Terminal Session
- [ ] Resumable Sessions
- [ ] Acts as a bridge between WebSocket connections and SSH protocol.
- [ ] Supports secure and encrypted communication.
- [ ] Lightweight and designed for high performance.
- [ ] Easy to deploy and integrate with existing systems.

## Installation

1. Clone the repository:

```bash
git clone https://github.com/talk2drys/shellcove.git
```

2. Build the project:

```bash
cd shellcove
cargo build --release
```

3. Run the service:

```bash
./target/release/shellcove
```

## Configuration
[comment]: <> (add configuration details)


## Usage

1. Start the Shellcove service:

```bash
./target/release/shellcove
```

2. Connect to the WebSocket server using a shell terminal client that supports WebSocket connections.

   Example using `websocat`:

   ```bash
   websocat ws://localhost:8080
   ```

## License

This project is licensed under the [MIT License](LICENSE).
