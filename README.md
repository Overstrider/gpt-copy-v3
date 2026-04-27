# GPT Copy v3

A CodeDungeon v3 example: a ChatGPT-style monorepo with a Rust Axum backend and a Next.js frontend.

## Setup

Copy environment placeholders and provide a local OpenRouter key outside git:

```powershell
Copy-Item .env.example .env
$env:OPENROUTER_API_KEY = "<your-openrouter-key>"
$env:OPENROUTER_MODEL = "openrouter/free"
```

Never commit `.env` or real provider keys.

## Backend

```powershell
Set-Location backend
cargo fmt --check
cargo test
cargo build
$env:DATABASE_URL = "sqlite://gpt-copy.db"
cargo run
```

The backend listens on `http://localhost:8080` by default and exposes:

- `GET /health`
- `GET /api/conversations`
- `POST /api/conversations`
- `GET /api/conversations/:id/messages`
- `POST /api/chat`
- `POST /api/chat/stream`

## Frontend

```powershell
Set-Location frontend
npm ci
npm audit --omit=dev
npm run lint
npm run typecheck
npm test
npm run build
npm run test:e2e
npm run dev
```

The frontend reads `NEXT_PUBLIC_API_BASE_URL`, defaulting to `http://localhost:8080`.

## Troubleshooting

- If chat requests fail with a provider error, confirm `OPENROUTER_API_KEY` is present only in your shell or GitHub secrets.
- If the frontend cannot reach the backend, confirm `BACKEND_PORT` and `NEXT_PUBLIC_API_BASE_URL` match.
- Tests mock provider calls and do not require real OpenRouter credentials.
