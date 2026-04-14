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
