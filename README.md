# Scratchpad v2 - Deployment Orchestration for Git Branches

A modern, lightweight deployment orchestration tool written in Rust that creates isolated scratch environments for Git branches. No more complex scripts or manual configuration - just define what services you need and watch them spin up automatically.

## Why Scratchpad v2?

The original Scratchpad used bash scripts, Node.js Express, and React with high complexity. Scratchpad v2 simplifies everything:

- **Single configuration file** (`scratchpad.toml`) replaces all bash scripts
- **Pure Rust CLI + HTTP API** - fast, reliable, no runtime dependencies
- **Minimal web UI** - uses htmx instead of React
- **Auto-managed Docker** - automatic network, volume, and service management
- **Built-in nginx routing** - subdomain or path-based routing out of the box
- **Shared services** - optional PostgreSQL, Redis, Kafka shared across scratches
- **Per-scratch services** - or dedicate services to individual scratches

## Quick Start

### Installation

```bash
# Build from source
cargo build --release
./target/release/scratchpad --version
```

### Basic Setup

1. **Initialize configuration:**
   ```bash
   scratchpad init
   ```
   This creates `scratchpad.toml` with sensible defaults.

2. **Edit configuration** (optional):
   ```bash
   cat scratchpad.toml.example  # See example configuration
   # Customize services, docker settings, nginx routing, etc.
   ```

3. **Create your first scratch from a branch:**
   ```bash
   scratchpad create --branch feature/my-feature
   ```

4. **View your scratches:**
   ```bash
   scratchpad list
   ```

5. **Start the HTTP API and web UI:**
   ```bash
   scratchpad serve
   # Access at http://localhost:3456
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

# Start a stopped scratch environment
scratchpad start <NAME>

# Stop a running scratch environment
scratchpad stop <NAME>

# Restart a scratch environment
scratchpad restart <NAME>

# Delete a scratch environment (with confirmation)
scratchpad delete <NAME>

# View logs from a scratch environment
scratchpad logs <NAME> [--service <SERVICE>] [--tail <N>]
```

### Server & API

```bash
# Start the HTTP API server and web UI
scratchpad serve [--port <PORT>]
```

### Services Management

```bash
# View status of shared services
scratchpad services list

# Start all shared services
scratchpad services start

# Stop all shared services
scratchpad services stop
```

### Nginx Configuration

```bash
# Generate nginx configuration
scratchpad nginx generate

# Reload nginx configuration (requires docker or manual setup)
scratchpad nginx reload

# View current nginx config
scratchpad nginx show
```

## Configuration

### Example scratchpad.toml

```toml
[server]
host = "0.0.0.0"
port = 3456
releases_dir = "./releases"

[docker]
socket = "/var/run/docker.sock"
network = "scratchpad-network"
label_prefix = "scratchpad"

[nginx]
enabled = true
config_path = "./nginx/scratches.conf"
domain = "scratches.localhost"
routing = "subdomain"  # or "path"

# Shared services available to all scratches
[services.postgres]
image = "postgres:18"
shared = true
port = 5432
env = { POSTGRES_PASSWORD = "postgres", POSTGRES_USER = "postgres" }
healthcheck = "pg_isready -U postgres"

# Per-scratch services (one per scratch)
[services.redis]
image = "redis:8-alpine"
shared = false
healthcheck = "redis-cli ping"

[scratch.defaults]
template = "default"
services = ["postgres", "redis"]

# Profiles for different scratch types
[scratch.profiles.minimal]
services = ["postgres"]

[scratch.profiles.full]
services = ["postgres", "redis"]
```

### Configuration Options

#### Server
- `host`: Bind address for API server (default: `0.0.0.0`)
- `port`: Port for API server (default: `3456`)
- `releases_dir`: Directory where scratch environments are stored (default: `./releases`)

#### Docker
- `socket`: Path to Docker socket (default: `/var/run/docker.sock`)
- `network`: Docker network name for scratches (default: `scratchpad-network`)
- `label_prefix`: Prefix for Docker labels (default: `scratchpad`)

#### Nginx
- `enabled`: Enable nginx routing (default: `true`)
- `config_path`: Path to generated nginx config
- `domain`: Base domain for routing (e.g., `scratches.localhost`)
- `routing`: `subdomain` or `path` based routing
- `container`: (optional) Nginx container name for automatic reload
- `reload_command`: (optional) Custom command to reload nginx

#### Services
Each service can be configured with:
- `image`: Docker image to use
- `shared`: `true` for shared service, `false` for per-scratch
- `port`: Port number
- `env`: Environment variables as inline TOML table
- `healthcheck`: Health check command

## Architecture

### File Structure

```
scratchpad/
├── Cargo.toml
├── scratchpad.toml          # Your configuration
├── scratchpad.toml.example  # Example configuration
├── releases/                # Created scratch directories
├── nginx/                   # Generated nginx configs
├── logs/                    # Scratch environment logs
└── src/
    ├── main.rs              # CLI entry point
    ├── error.rs             # Error handling
    ├── config/              # Configuration loading
    ├── cli/                 # CLI command implementations
    ├── docker/              # Docker client and operations
    ├── scratch/             # Scratch lifecycle management
    ├── services/            # Shared services (postgres, redis, etc)
    ├── nginx/               # Nginx config generation
    ├── api/                 # HTTP API server
    └── ui/                  # Web UI handlers
```

### How It Works

1. **Configuration Load**: `scratchpad.toml` is parsed to configure server, docker, nginx, and services
2. **Docker Network**: Creates a Docker network for all scratches to communicate
3. **Scratch Creation**:
   - Generates unique name from branch (sanitized)
   - Creates docker-compose.yml with configured services
   - Starts containers with labels for tracking
   - Creates nginx config for routing
   - Reloads nginx
4. **Service Provisioning**:
   - Shared services (postgres, redis) run once and are reused
   - Per-scratch services get their own containers
   - All services get health checks for auto-restart
5. **Routing**:
   - Subdomain routing: `feature-my-branch.scratches.localhost`
   - Path routing: `scratches.localhost/feature-my-branch`

## HTTP API

When running `scratchpad serve`, the following REST endpoints are available:

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

## GitHub Webhook Integration

Configure a GitHub webhook to automatically create scratches for pull requests:

1. Go to your GitHub repository settings
2. Add webhook: `http://your-domain:3456/webhook/github`
3. Select "Push events" and "Pull request events"
4. Scratchpad will automatically create/update scratches

## Advanced Usage

### Custom Docker Compose

Define custom docker-compose services in your scratch profiles:

```toml
[scratch.profiles.custom]
services = ["postgres"]
# Custom docker compose will be merged with generated one
```

### Environment Variables

Use environment variable interpolation in config:

```toml
[services.postgres]
env = { POSTGRES_PASSWORD = "${DB_PASSWORD}" }
# Will use DB_PASSWORD environment variable
```

### Multiple Profiles

Switch between different service setups:

```bash
# Create with minimal services
scratchpad create --branch feature/api --profile minimal

# Create with full services
scratchpad create --branch feature/web --profile full
```

## Troubleshooting

### Docker connection refused
Make sure Docker socket is accessible:
```bash
ls -la /var/run/docker.sock
# Fix permissions if needed
sudo usermod -aG docker $USER
```

### Nginx reload fails
If nginx is not in Docker, configure custom reload:
```toml
[nginx]
reload_command = "sudo systemctl reload nginx"
# or
reload_command = "docker exec nginx nginx -s reload"
```

### Services not starting
Check service logs:
```bash
scratchpad logs <scratch-name> --service postgres
```

### Port already in use
Scratches use dynamic ports. Ensure enough ports are available:
```bash
# Check what ports are in use
docker ps --format "table {{.Names}}\t{{.Ports}}"
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

### Code Structure

- **Error Handling**: Custom error types in `src/error.rs`
- **Configuration**: TOML parsing with environment variable support
- **Docker**: Bollard client wrapper with async operations
- **CLI**: Clap-based command parsing
- **API**: Axum web framework
- **Templates**: MiniJinja for docker-compose rendering

## Migration from v1

If you're using the original Scratchpad:

1. The new format uses `scratchpad.toml` instead of multiple bash scripts
2. Services are defined in TOML instead of docker-compose templates
3. All Docker operations are handled automatically
4. The CLI is more powerful but simpler to use

## Performance

- **Init time**: ~500ms to generate config
- **Scratch creation**: ~2-3s (depends on image pull speed)
- **List operation**: ~1-2s (depends on Docker daemon)
- **API response**: <100ms for typical operations

## License

MIT

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Submit a pull request with tests

## Support

- GitHub Issues: Report bugs and request features
- GitHub Discussions: Ask questions and share ideas
