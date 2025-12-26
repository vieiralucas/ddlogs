# ddlogs

A CLI tool for tailing and querying Datadog logs, similar to `tail -f` for your Datadog logs.

## Features

- 🔍 Query logs with filters (service, source, host)
- 👀 Follow mode for real-time log tailing
- ⚙️ Config file support for storing credentials
- 🌍 Multi-region Datadog support
- 📊 Single-line JSON output per log
- ⏱️ Respects Datadog API rate limits

## Installation

### Using Cargo

```bash
cargo install ddlogs
```

### Using install script

```bash
curl -fsSL https://raw.githubusercontent.com/vieiralucas/ddlogs/main/install.sh | sh
```

### Download pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/vieiralucas/ddlogs/releases).

## Configuration

First, configure your Datadog credentials:

```bash
ddlogs configure
```

This will prompt you for:
- **Datadog API Key**
- **Datadog Application Key**
- **Datadog Site** (default: `datadoghq.com`)

Configuration is saved to `~/.config/ddlogs/config.toml`.

Alternatively, you can set environment variables:
```bash
export DD_API_KEY=your_api_key
export DD_APP_KEY=your_app_key
export DD_SITE=us5.datadoghq.com  # optional, defaults to datadoghq.com
```

## Usage

### Basic log query (last hour)

```bash
ddlogs
```

### Filter by service

```bash
ddlogs --service web-api
```

### Filter by multiple criteria

```bash
ddlogs --service nginx --host prod-01 --limit 50
```

### Custom Datadog query

```bash
ddlogs --query "status:error"
ddlogs --query "service:nginx AND status:error"
```

### Follow mode (like `tail -f`)

```bash
ddlogs --follow
ddlogs -f --service email-api
```

### Custom polling interval

```bash
# Poll every 15 seconds instead of default 12
ddlogs -f --interval 15
```

### Limit number of results

```bash
ddlogs --limit 50
```

### Pipe to jq for filtering

```bash
ddlogs --service api | jq -r '.content.message'
```

## Options

```
Usage: ddlogs [OPTIONS] [COMMAND]

Commands:
  configure  Configure ddlogs with API credentials and site
  help       Print this message or the help of the given subcommand(s)

Options:
  -f, --follow               Follow mode - continuously poll for new logs
      --service <SERVICE>    Filter by service
      --source <SOURCE>      Filter by source
      --host <HOST>          Filter by host
  -q, --query <QUERY>        Raw Datadog query string
  -l, --limit <LIMIT>        Number of logs to retrieve [default: 100]
      --interval <INTERVAL>  Poll interval in seconds for follow mode [default: 12]
  -h, --help                 Print help
```

## Rate Limits

ddlogs respects Datadog's API rate limits:
- Default polling interval: **12 seconds** (300 requests/hour)
- Datadog allows **2 requests per 10 seconds** for log queries
- Adjust `--interval` if you hit rate limits

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy
```

## License

Apache-2.0 License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
