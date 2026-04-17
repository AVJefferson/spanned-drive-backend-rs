# SPANNED DRIVE RUST BACKEND

## Commands

### Start Server Using Docker

```bash
# dev
clear && docker compose --file docker/docker-compose.yml --profile local up -d
```

```bash
# dev
clear && docker compose --file docker/docker-compose.yml --profile dev up --build --watch
```

```bash
# stg
clear && docker compose --file docker/docker-compose.yml --profile stg up --build
```

```bash
# prd
clear && docker compose --file docker/docker-compose.yml --profile prd up --build
```

### Start Server Directly

```bash
# dev
clear && set -a && source local.env && set +a && cargo dev
```

```bash
# stg
clear && cargo stg
```

```bash
# prd
clear && docker prd
```

### Build Application

```bash
# dev
clear && cargo build-dev
```

```bash
# stg
clear && cargo build-stg
```

```bash
# prd
clear && docker build-prd
```

### Test Application

```bash
# dev
clear && set -a && source local.env && set +a && cargo test-dev
```

## PreRequisites

### Install Docker 
[Install from Docker Official Website](https://docs.docker.com/desktop/setup/install/linux/)

### Install Rust
[Install from Rust Official Website](https://www.rust-lang.org/tools/install)

### Create allowed_clients directory and create a new key file
```bash
mkdir -p allowed_clients && touch allowed_clients/test.key
echo '{"token": "test", "permissions": ["test"]}' > allowed_clients/test.key
```

### Create Environment Variables File {local, dev, stg, prd}.env
```bash
ENVIRONMENT=local
SERVER_PORT=3000
SERVER_HOST=0.0.0.0

ENABLE_EXTERNAL_SYSTEM_GOOGLE=true
GOOGLE_CLIENT_ID="<your-google-client-id>.apps.googleusercontent.com"
GOOGLE_CLIENT_SECRET=<your-google-client-secret>
```


## Other Helpful Commands

### Cargo Audit
```bash
cargo install cargo-audit
clear && cargo audit
clear && cargo audit fix
```

### Cargo crates Features

```bash
cargo install cargo-features
clear && cargo features
```
