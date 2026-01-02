# Installation

Detailed installation instructions for all platforms.

---

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://rcurl.dev/install.sh | bash
```

With options:

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

With options:

```powershell
# Specific version
$env:RCURL_VERSION = "v0.1.0"; irm https://rcurl.dev/install.ps1 | iex
```

---

## Package Managers

### Homebrew (macOS / Linux)

```bash
# Add the tap
brew tap rcurl/tap

# Install
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

# Install
scoop install rcurl
```

### AUR (Arch Linux)

```bash
# Using yay
yay -S rcurl

# Or using paru
paru -S rcurl
```

---

## Docker

### Quick Run

```bash
# Single request
docker run --rm ghcr.io/user/rcurl https://example.com

# Save output to local directory
docker run --rm -v $(pwd)/output:/output ghcr.io/user/rcurl \
    https://example.com -o /output/result.html
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  rcurl:
    image: ghcr.io/user/rcurl
    volumes:
      - ./output:/output
```

```bash
docker-compose run --rm rcurl https://example.com
```

---

## Manual Installation

### Download

Download from [GitHub Releases](https://github.com/user/rcurl/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `rcurl-linux-x86_64.tar.gz` |
| Linux aarch64 | `rcurl-linux-aarch64.tar.gz` |
| macOS x86_64 | `rcurl-darwin-x86_64.tar.gz` |
| macOS aarch64 | `rcurl-darwin-aarch64.tar.gz` |
| Windows x86_64 | `rcurl-windows-x86_64.zip` |

### Directory Structure

After extraction:

```
rcurl/
├── rcurl(.exe)              # Main binary
├── rcurld(.exe)             # Daemon binary
└── bin/
    ├── curl_engine(.exe)    # Upstream curl
    ├── curl_chrome          # Chrome impersonation (Linux/macOS)
    ├── curl_ff              # Firefox impersonation (Linux/macOS)
    └── curl_safari          # Safari impersonation (Linux/macOS)
```

### Linux (Manual)

```bash
# Extract
tar -xzf rcurl-linux-x86_64.tar.gz
sudo mv rcurl /usr/local/

# Add to PATH
echo 'export PATH="/usr/local/rcurl:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Optional: alias curl
echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.bashrc
source ~/.bashrc
```

### macOS (Manual)

```bash
# Extract
tar -xzf rcurl-darwin-aarch64.tar.gz
sudo mv rcurl /usr/local/

# Add alias
echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.zshrc
source ~/.zshrc
```

### Windows (Manual)

1. Extract `rcurl-windows-x86_64.zip` to `C:\Program Files\rcurl\`

2. Add to PATH:
    - Open System Properties → Environment Variables
    - Edit `Path` under User variables
    - Add `C:\Program Files\rcurl`

3. Set up alias in PowerShell profile (`$PROFILE`):

    ```powershell
    Set-Alias -Name curl -Value "C:\Program Files\rcurl\rcurl.exe" -Option AllScope
    ```

---

## Building from Source

### Prerequisites

- Rust 1.75 or later
- pkg-config and OpenSSL headers (Linux)

### Build

=== "Linux"

    ```bash
    # Install dependencies (Debian/Ubuntu)
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev

    # Clone and build
    git clone https://github.com/user/rcurl.git
    cd rcurl
    cargo build --release

    # Binary location
    ./target/release/rcurl --version
    ```

=== "macOS"

    ```bash
    # Install Xcode tools
    xcode-select --install

    # Clone and build
    git clone https://github.com/user/rcurl.git
    cd rcurl
    cargo build --release
    ```

=== "Windows"

    ```powershell
    # Requires Visual Studio Build Tools with C++ workload
    git clone https://github.com/user/rcurl.git
    cd rcurl
    cargo build --release

    # Binary location
    .\target\release\rcurl.exe --version
    ```

### Run Tests

```bash
# All tests
cargo test

# Browser integration tests (requires Chromium)
cargo test --test browser_integration -- --test-threads=1
```

---

## Chromium Auto-Download

rcurl automatically downloads Chromium on first JS preflight use. No manual installation required.

**Cache locations:**

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/rcurl/chromium/` |
| macOS | `~/Library/Application Support/rcurl/chromium/` |
| Windows | `%LOCALAPPDATA%\rcurl\chromium\` |

!!! note "Linux ARM64"
    Auto-download is not available for Linux ARM64. Install Chromium manually:

    ```bash
    # Ubuntu/Debian
    sudo apt install chromium-browser

    # Fedora
    sudo dnf install chromium

    # Arch
    sudo pacman -S chromium
    ```

---

## Verify Installation

```bash
# Check version
rcurl --version

# Test with debug output
rcurl --rcurl-debug https://httpbin.org/get

# Test JS preflight (downloads Chromium if needed)
rcurl --rcurl-js --rcurl-debug https://example.com
```

---

## Uninstall

### Linux / macOS

```bash
# Remove alias from shell config, then:
sudo rm -rf /usr/local/rcurl

# Remove Chromium cache
rm -rf ~/.local/share/rcurl  # Linux
rm -rf ~/Library/Application\ Support/rcurl  # macOS
```

### Windows

1. Remove from PATH in Environment Variables
2. Delete `C:\Program Files\rcurl\`
3. Remove alias from PowerShell profile
4. Delete `%LOCALAPPDATA%\rcurl\`
