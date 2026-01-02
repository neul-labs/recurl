# Quick Start

Get rcurl running in under 2 minutes.

---

## Install

=== "Linux / macOS"

    ```bash
    curl -fsSL https://rcurl.dev/install.sh | bash
    ```

=== "Windows (PowerShell)"

    ```powershell
    irm https://rcurl.dev/install.ps1 | iex
    ```

=== "Homebrew"

    ```bash
    brew install rcurl/tap/rcurl
    ```

=== "Docker"

    ```bash
    docker run --rm ghcr.io/user/rcurl https://example.com
    ```

---

## Verify Installation

```bash
rcurl --version
```

You should see the curl version info (rcurl uses a real curl engine).

---

## Make Your First Request

```bash
# Basic request
rcurl https://httpbin.org/get

# With debug output to see what's happening
rcurl --rcurl-debug https://httpbin.org/get
```

---

## Set Up the curl Alias (Optional)

The installer will ask if you want to alias `curl` to `rcurl`. If you said no but want to enable it later:

=== "Bash"

    ```bash
    echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.bashrc
    source ~/.bashrc
    ```

=== "Zsh"

    ```bash
    echo 'alias curl="/usr/local/rcurl/rcurl"' >> ~/.zshrc
    source ~/.zshrc
    ```

=== "PowerShell"

    ```powershell
    Add-Content $PROFILE 'Set-Alias -Name curl -Value "C:\Program Files\rcurl\rcurl.exe"'
    . $PROFILE
    ```

Now `curl` commands automatically use rcurl:

```bash
curl https://protected-site.com  # Uses rcurl transparently
```

---

## Test Anti-Bot Bypass

Try a site that blocks regular curl:

```bash
# This might be blocked with regular curl
rcurl --rcurl-debug https://nowsecure.nl

# Output shows the escalation:
# [rcurl] curl_engine: 403 Cloudflare
# [rcurl] Escalating: impersonation (chrome)
# [rcurl] curl_chrome: 200 OK
```

---

## What's Next?

- [Installation Details](installation.md) - Platform-specific options
- [CLI Reference](../usage/cli.md) - All available flags
- [How It Works](../how-it-works/architecture.md) - Under the hood
