# Installation

## Quick install (recommended)

### Linux / macOS

```bash
curl -fsSL https://recurl.dev/install.sh | bash
```

Or with options:

```bash
# Specific version
RCURL_VERSION=v0.1.2 curl -fsSL https://recurl.dev/install.sh | bash

# Custom install directory
RCURL_INSTALL_DIR=/opt/recurl curl -fsSL https://recurl.dev/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://recurl.dev/install.ps1 | iex
```

Or with options:

```powershell
# Specific version
$env:RCURL_VERSION = "v0.1.2"; irm https://recurl.dev/install.ps1 | iex

# Custom install directory
& { param($InstallDir) irm https://recurl.dev/install.ps1 | iex } -InstallDir "D:\tools\recurl"
```

## What the installer does

1. Detects your platform and architecture
2. Downloads the appropriate binary archive
3. Extracts to install directory:
   - Linux/macOS: `/usr/local/recurl` (or `~/.local/recurl` if no write permission)
   - Windows: `%LOCALAPPDATA%\recurl`
4. **Asks** if you want to set up a `curl` alias (optional, with your consent)
5. Adds alias to your shell config if you agree

## Manual installation

If you prefer not to use the installer:

### Download

Download the appropriate archive from [GitHub Releases](https://github.com/neul-labs/recurl/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `recurl-linux-x86_64.tar.gz` |
| Linux aarch64 | `recurl-linux-aarch64.tar.gz` |
| macOS x86_64 | `recurl-darwin-x86_64.tar.gz` |
| macOS aarch64 (Apple Silicon) | `recurl-darwin-aarch64.tar.gz` |
| Windows x86_64 | `recurl-windows-x86_64.zip` |

### Directory structure

After extraction:

```
recurl/
├── recurl(.exe)              # main binary
├── recurld(.exe)             # daemon binary
└── bin/
    ├── curl_engine(.exe)    # upstream curl
    ├── curl_chrome(.exe)    # chrome impersonation (Linux/macOS only)
    ├── curl_ff(.exe)        # firefox impersonation (Linux/macOS only)
    └── curl_safari(.exe)    # safari impersonation (Linux/macOS only)
```

### Linux (manual)

```bash
# Extract
tar -xzf recurl-linux-x86_64.tar.gz
sudo mv recurl /usr/local/

# Add alias to shell config
echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.bashrc
source ~/.bashrc
```

### macOS (manual)

```bash
# Extract
tar -xzf recurl-darwin-aarch64.tar.gz
sudo mv recurl /usr/local/

# Add alias to shell config
echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.zshrc
source ~/.zshrc
```

### Windows (manual)

1. Extract `recurl-windows-x86_64.zip` to `C:\Program Files\recurl\`

2. Add to PATH (optional):
   - Open System Properties → Environment Variables
   - Edit `Path` under User variables
   - Add `C:\Program Files\recurl`

3. Set up alias:

**PowerShell** (add to `$PROFILE`):
```powershell
Set-Alias -Name curl -Value "C:\Program Files\recurl\recurl.exe" -Option AllScope
```

**Git Bash** (add to `~/.bashrc`):
```bash
alias curl='/c/Program\ Files/recurl/recurl.exe'
```

## Package managers

### Homebrew (macOS/Linux)

```bash
# Add the tap
brew tap neul-labs/tap

# Install recurl
brew install recurl
```

Or install directly:
```bash
brew install neul-labs/tap/recurl
```

### npm (Node.js)

```bash
# Global install
npm install -g recurl-cli

# Or run without installing
npx recurl-cli https://example.com
```

### PyPI (Python)

```bash
# Install
pip install recurl-cli

# Run
python -m recurl https://example.com
```

### Scoop (Windows)

```powershell
# Add the bucket
scoop bucket add recurl https://github.com/neul-labs/recurl

# Install recurl
scoop install recurl
```

## Docker

### Quick run

```bash
# Run a single request
docker run --rm ghcr.io/neul-labs/recurl https://example.com

# Run with output to local directory
docker run --rm -v $(pwd)/output:/output ghcr.io/neul-labs/recurl https://example.com -o /output/result.html
```

### Docker Compose

```bash
# Clone the repository
git clone https://github.com/neul-labs/recurl.git
cd recurl

# Run a single request
docker-compose run --rm recurl https://example.com

# Start the daemon in background
docker-compose up -d recurld
```

### Build locally

```bash
# Build the image
docker build -t recurl .

# Run
docker run --rm recurl https://example.com
```

## Building from source

### Prerequisites

- Rust 1.75 or later
- pkg-config and OpenSSL development headers (Linux)
- For JS preflight: Chromium/Chrome installed

### Linux

```bash
# Install dependencies (Debian/Ubuntu)
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev chromium

# Clone and build
git clone https://github.com/neul-labs/recurl.git
cd recurl
cargo build --release

# Build daemon
cargo build --release --bin recurld --features daemon

# Binaries are in target/release/
./target/release/recurl --version
./target/release/recurld --version
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Clone and build
git clone https://github.com/neul-labs/recurl.git
cd recurl
cargo build --release
cargo build --release --bin recurld --features daemon

# Binaries are in target/release/
./target/release/recurl --version
```

### Windows

```powershell
# Requires Visual Studio Build Tools with C++ workload
# Clone and build
git clone https://github.com/neul-labs/recurl.git
cd recurl
cargo build --release

# Binary is in target\release\
.\target\release\recurl.exe --version
```

**Note**: The daemon (recurld) is not fully supported on Windows yet.

### Run tests

```bash
# Run all tests
cargo test

# Run with specific test
cargo test test_smart_mode

# Run conformance tests
cargo test conformance
```

## Verify installation

```bash
# Should show recurl executing curl_engine
recurl --version

# If alias is set up:
curl --version

# Test smart fallback
recurl --recurl-debug https://example.com
```

## Platform limitations

### Windows

- **No impersonation layer**: curl-impersonate is not available for Windows. On Windows, recurl skips directly from `curl_engine` to JS preflight on failure.
- **Daemon uses named pipes**: Instead of Unix sockets, the daemon uses `\\.\pipe\recurl-<username>` on Windows.

### Escalation behavior by platform

| Platform | Layer 1 (curl_engine) | Layer 2 (Impersonation) | Layer 3 (JS preflight) |
|----------|----------------------|------------------------|----------------------|
| Linux | Yes | Yes | Yes |
| macOS | Yes | Yes | Yes |
| Windows | Yes | **No** (skipped) | Yes |

## Uninstall

### Linux/macOS

```bash
# Remove alias from shell config
# Then:
sudo rm -rf /usr/local/recurl
sudo rm -f /usr/local/bin/curl  # if symlink was created
```

### Windows

1. Remove from PATH in Environment Variables
2. Delete `C:\Program Files\recurl\`
3. Remove alias from PowerShell profile
