# Installation

## Quick install (recommended)

### Linux / macOS

```bash
curl -fsSL https://rcurl.dev/install.sh | bash
```

Or with options:

```bash
# Specific version
RCURL_VERSION=v0.1.0 curl -fsSL https://rcurl.dev/install.sh | bash

# Custom install directory
RCURL_INSTALL_DIR=/opt/rcurl curl -fsSL https://rcurl.dev/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://rcurl.dev/install.ps1 | iex
```

Or with options:

```powershell
# Specific version
$env:RCURL_VERSION = "v0.1.0"; irm https://rcurl.dev/install.ps1 | iex

# Custom install directory
& { param($InstallDir) irm https://rcurl.dev/install.ps1 | iex } -InstallDir "D:\tools\rcurl"
```

## What the installer does

1. Detects your platform and architecture
2. Downloads the appropriate binary archive
3. Extracts to install directory:
   - Linux/macOS: `/usr/local/rcurl` (or `~/.local/rcurl` if no write permission)
   - Windows: `%LOCALAPPDATA%\rcurl`
4. **Asks** if you want to set up a `curl` alias (optional, with your consent)
5. Adds alias to your shell config if you agree

## Manual installation

If you prefer not to use the installer:

### Download

Download the appropriate archive from [GitHub Releases](https://github.com/user/rcurl/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `rcurl-linux-x86_64.tar.gz` |
| Linux aarch64 | `rcurl-linux-aarch64.tar.gz` |
| macOS x86_64 | `rcurl-darwin-x86_64.tar.gz` |
| macOS aarch64 (Apple Silicon) | `rcurl-darwin-aarch64.tar.gz` |
| Windows x86_64 | `rcurl-windows-x86_64.zip` |

### Directory structure

After extraction:

```
rcurl/
├── rcurl(.exe)              # main binary
├── rcurld(.exe)             # daemon binary
└── bin/
    ├── curl_engine(.exe)    # upstream curl
    ├── curl_chrome(.exe)    # chrome impersonation (Linux/macOS only)
    ├── curl_ff(.exe)        # firefox impersonation (Linux/macOS only)
    └── curl_safari(.exe)    # safari impersonation (Linux/macOS only)
```

### Linux (manual)

```bash
# Extract
tar -xzf rcurl-linux-x86_64.tar.gz
sudo mv rcurl /usr/local/

# Add alias to shell config
echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.bashrc
source ~/.bashrc
```

### macOS (manual)

```bash
# Extract
tar -xzf rcurl-darwin-aarch64.tar.gz
sudo mv rcurl /usr/local/

# Add alias to shell config
echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.zshrc
source ~/.zshrc
```

### Windows (manual)

1. Extract `rcurl-windows-x86_64.zip` to `C:\Program Files\rcurl\`

2. Add to PATH (optional):
   - Open System Properties → Environment Variables
   - Edit `Path` under User variables
   - Add `C:\Program Files\rcurl`

3. Set up alias:

**PowerShell** (add to `$PROFILE`):
```powershell
Set-Alias -Name curl -Value "C:\Program Files\rcurl\rcurl.exe" -Option AllScope
```

**Git Bash** (add to `~/.bashrc`):
```bash
alias curl='/c/Program\ Files/rcurl/rcurl.exe'
```

## Package managers

### Homebrew (macOS/Linux)

```bash
# Add the tap
brew tap rcurl/tap

# Install rcurl
brew install rcurl
```

Or install directly:
```bash
brew install rcurl/tap/rcurl
```

### Scoop (Windows)

```powershell
# Add the bucket
scoop bucket add rcurl https://github.com/user/rcurl

# Install rcurl
scoop install rcurl
```

## Docker

### Quick run

```bash
# Run a single request
docker run --rm ghcr.io/user/rcurl https://example.com

# Run with output to local directory
docker run --rm -v $(pwd)/output:/output ghcr.io/user/rcurl https://example.com -o /output/result.html
```

### Docker Compose

```bash
# Clone the repository
git clone https://github.com/user/rcurl.git
cd rcurl

# Run a single request
docker-compose run --rm rcurl https://example.com

# Start the daemon in background
docker-compose up -d rcurld
```

### Build locally

```bash
# Build the image
docker build -t rcurl .

# Run
docker run --rm rcurl https://example.com
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
git clone https://github.com/user/rcurl.git
cd rcurl
cargo build --release

# Build daemon
cargo build --release --bin rcurld --features daemon

# Binaries are in target/release/
./target/release/rcurl --version
./target/release/rcurld --version
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Clone and build
git clone https://github.com/user/rcurl.git
cd rcurl
cargo build --release
cargo build --release --bin rcurld --features daemon

# Binaries are in target/release/
./target/release/rcurl --version
```

### Windows

```powershell
# Requires Visual Studio Build Tools with C++ workload
# Clone and build
git clone https://github.com/user/rcurl.git
cd rcurl
cargo build --release

# Binary is in target\release\
.\target\release\rcurl.exe --version
```

**Note**: The daemon (rcurld) is not fully supported on Windows yet.

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
# Should show rcurl executing curl_engine
rcurl --version

# If alias is set up:
curl --version

# Test smart fallback
rcurl --rcurl-debug https://example.com
```

## Platform limitations

### Windows

- **No impersonation layer**: curl-impersonate is not available for Windows. On Windows, rcurl skips directly from `curl_engine` to JS preflight on failure.
- **Daemon uses named pipes**: Instead of Unix sockets, the daemon uses `\\.\pipe\rcurl-<username>` on Windows.

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
sudo rm -rf /usr/local/rcurl
sudo rm -f /usr/local/bin/curl  # if symlink was created
```

### Windows

1. Remove from PATH in Environment Variables
2. Delete `C:\Program Files\rcurl\`
3. Remove alias from PowerShell profile
