# Scratchpad v2 - Deployment Orchestration for Git Branches

A modern, lightweight deployment orchestration tool written in Rust that creates isolated scratch environments for Git branches. No more complex scripts or manual configuration - just define what services you need and watch them spin up automatically.

## Why Scratchpad v2?

The original Scratchpad used bash scripts, Node.js Express, and React with high complexity. Scratchpad v2 simplifies everything:

- **Single configuration file** (`scratchpad.toml`) replaces all bash scripts
- **Pure Rust CLI + HTTP API** - fast, reliable, no runtime dependencies
- **Minimal web UI** - uses htmx instead of React
- **Auto-managed Docker** - automatic network, volume, and service management
- **Built-in nginx routing** - dynamic wildcard routing, no reload needed per scratch
- **Shared services** - PostgreSQL, Redis, Nginx shared across all scratches
- **Per-scratch services** - your app containers, one set per scratch

## Quick Start

### Installation

**One-liner install (recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/Krakaw/scratchpad/main/install.sh | bash
```

**Using cargo:**

```bash
cargo install --git https://github.com/Krakaw/scratchpad
```

**Build from source:**

```bash
git clone https://github.com/Krakaw/scratchpad.git
cd scratchpad
cargo build --release
./target/release/scratchpad --version
```

### Interactive Setup (Recommended)

```bash
scratchpad setup
```

This walks you through:
- Docker socket detection
- Domain and routing configuration
- Service selection (PostgreSQL, Redis, Nginx, etc.)
- Creates a ready-to-use `scratchpad.toml`

### Manual Setup

1. **Initialize configuration:**
   ```bash
   scratchpad init
   ```

2. **Edit `scratchpad.toml`** to configure services and nginx

3. **Start shared services:**
   ```bash
   scratchpad services start
   ```

4. **Create your first scratch:**
   ```bash
   scratchpad create --branch feature/my-feature
   ```

## CLI Commands

### Scratch Management

```bash
# Create a new scratch environment from a branch
scratchpad create --branch <BRANCH> [--name <NAME>] [--profile <PROFILE>]

# List all scratch environments
scratchpad list

# Get detailed status of a scratch
scratchpad status <NAME>

# Update a scratch (regenerate compose.yml from current config)
scratchpad update <NAME> [--restart]

# Start/stop/restart a scratch
scratchpad start <NAME>
scratchpad stop <NAME>
scratchpad restart <NAME>

# Delete a scratch environment
scratchpad delete <NAME> [--force]

# View logs
scratchpad logs <NAME> [--service <SERVICE>] [--follow] [--tail <N>]
```

### Services Management

```bash
# Start all shared services (postgres, redis, nginx)
scratchpad services start

# Stop all shared services
scratchpad services stop

# Restart shared services with current config
scratchpad services restart

# View status of shared services
scratchpad services status

# Remove all shared service containers (for config changes)
scratchpad services clean [--force]
```

### Nginx Configuration

```bash
# Generate/regenerate nginx configuration
scratchpad nginx generate

# Reload nginx
scratchpad nginx reload

# View current nginx config
scratchpad nginx show
```

### Configuration Management

```bash
# Validate configuration and show summary
scratchpad config check

# Display current config file
scratchpad config show
```

### System Health

```bash
# Check Docker connectivity and system health
scratchpad doctor
```

### Server & API

```bash
# Start the HTTP API server and web UI
scratchpad serve [--host <HOST>] [--port <PORT>]
```

## Configuration

### Example scratchpad.toml

```toml
[server]
host = "0.0.0.0"
port = 3456
releases_dir = "./releases"

[docker]
socket = "/var/run/docker.sock"  # or ~/.orbstack/run/docker.sock on macOS
network = "scratchpad-network"
label_prefix = "scratchpad"

[nginx]
enabled = true
config_path = "./nginx/scratches.conf"
domain = "scratch.local"
routing = "subdomain"        # or "path"
dynamic = true               # use wildcard routing (no reload per scratch)
ingress_service = "api"      # which service handles incoming requests

# Shared services (one instance, all scratches connect to it)
[services.postgres]
image = "postgres:16"
shared = true
port = 5432                  # host port
internal_port = 5432         # container port
env = { POSTGRES_PASSWORD = "postgres", POSTGRES_USER = "postgres" }
healthcheck = "pg_isready -U postgres"
auto_create_db = true        # create scratch_<name> database per scratch

[services.redis]
image = "redis:7-alpine"
shared = true
port = 6379
healthcheck = "redis-cli ping"

[services.nginx]
image = "nginx:alpine"
shared = true
port = 80

# Per-scratch services (each scratch gets its own)
[services.api]
image = "myorg/api:latest"
shared = false
internal_port = 3000
healthcheck = "curl -f http://localhost:3000/health"
[services.api.env]
NODE_ENV = "development"
# DATABASE_URL and REDIS_URL are auto-injected

[services.worker]
image = "myorg/worker:latest"
shared = false
[services.worker.env]
NODE_ENV = "development"

# Default services for new scratches
[scratch.defaults]
template = "default"
services = ["postgres", "redis", "nginx", "api", "worker"]

# Profiles for different scratch configurations
[scratch.profiles.minimal]
services = ["postgres", "api"]

[scratch.profiles.full]
services = ["postgres", "redis", "nginx", "api", "worker"]
```

### Key Configuration Options

#### Services

| Option | Description |
|--------|-------------|
| `image` | Docker image to use |
| `shared` | `true` = one instance for all scratches, `false` = per-scratch |
| `port` | Host port to expose |
| `internal_port` | Container port (defaults to host port or standard for known images) |
| `env` | Environment variables |
| `volumes` | Volume mounts |
| `healthcheck` | Health check command |
| `auto_create_db` | For postgres: create a database per scratch |

#### Nginx

| Option | Description |
|--------|-------------|
| `enabled` | Enable nginx routing |
| `domain` | Base domain (e.g., `scratch.local`) |
| `routing` | `subdomain` or `path` based routing |
| `dynamic` | Use wildcard routing (default: true) |
| `ingress_service` | Which service handles incoming requests |
| `container` | Container name for reload (auto-detected if using shared nginx) |

### Auto-Injected Environment Variables

Per-scratch services automatically receive:

- `DATABASE_URL` - Connection string to the scratch's postgres database
- `REDIS_URL` - Connection string to shared redis (`redis://scratchpad-redis:6379`)

## Architecture

### How It Works

1. **Shared Services**: PostgreSQL, Redis, Nginx run once and are shared
2. **Per-Scratch Services**: Your app containers, one set per scratch
3. **Database Isolation**: Each scratch gets its own database (`scratch_<name>`)
4. **Dynamic Routing**: Nginx routes based on subdomain/path without needing reload

### Routing

**Subdomain mode** (recommended):
```
feature-foo.scratch.local → feature-foo-api:3000
my-test.scratch.local → my-test-api:3000
```

**Path mode**:
```
scratch.local/feature-foo/ → feature-foo-api:3000
scratch.local/my-test/ → my-test-api:3000
```

### Container Naming

- Shared services: `scratchpad-<service>` (e.g., `scratchpad-postgres`)
- Per-scratch services: `<scratch>-<service>` (e.g., `feature-foo-api`)

## HTTP API

When running `scratchpad serve`:

```
GET  /health                    # Health check
GET  /scratches                 # List all scratches
POST /scratches                 # Create new scratch
GET  /scratches/:name           # Get scratch status
DELETE /scratches/:name         # Delete scratch
POST /scratches/:name/start     # Start scratch
POST /scratches/:name/stop      # Stop scratch
POST /scratches/:name/restart   # Restart scratch
GET  /scratches/:name/logs      # Get scratch logs

POST /webhook/github            # GitHub webhook receiver

GET  /services                  # List services status
POST /services/start            # Start all services
POST /services/stop             # Stop all services
```

## Troubleshooting

### Port already in use

If you change ports in config after containers exist:
```bash
scratchpad services clean
scratchpad services start
```

### Nginx not routing

1. Check `nginx.ingress_service` matches your service name
2. Verify the service has `internal_port` set
3. Run `scratchpad nginx generate` then `scratchpad nginx reload`

### Services not starting with scratch

Make sure services are in `scratch.defaults.services`:
```toml
[scratch.defaults]
services = ["postgres", "redis", "nginx", "api"]
```

### Docker connection refused

```bash
# Check socket exists
ls -la /var/run/docker.sock

# On macOS with OrbStack
ls -la ~/.orbstack/run/docker.sock

# Fix permissions if needed
sudo usermod -aG docker $USER
```

### Check configuration

```bash
scratchpad config check
scratchpad doctor
```

## Development

### Running Tests

```bash
cargo test
```

### Building Release

```bash
cargo build --release
```

## License

MIT
