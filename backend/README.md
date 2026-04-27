# gpt-copy-v3 Backend

Rust 2024 Axum API for the gpt-copy-v3 example.

## Environment

Copy `.env.example` values into your local shell or `.env` file. Do not commit real keys.

- `DATABASE_URL` defaults to `sqlite://gpt-copy.db?mode=rwc`
- `OPENROUTER_API_KEY` is required for live model calls
- `OPENROUTER_MODEL` defaults to `openrouter/free`
- `FRONTEND_ORIGIN` defaults to `http://localhost:3000`
- `BIND_ADDR` defaults to `127.0.0.1:8080`

## Commands

```powershell
cargo fmt
cargo test
cargo build
cargo run
```

## API

- `GET /health`
- `GET /api/conversations`
- `POST /api/conversations`
- `GET /api/conversations/:id/messages`
- `POST /api/chat`
- `POST /api/chat/stream`

Provider calls are proxied server-side through OpenRouter. Tests use a mock provider and do not require secrets.
