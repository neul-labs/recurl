#!/bin/bash
# Update package manifests with release checksums
# Usage: ./scripts/update-release.sh v0.1.0

set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.1.0"
    exit 1
fi

# Strip 'v' prefix if present
VERSION_NUM="${VERSION#v}"

echo "Updating package manifests for version $VERSION_NUM..."

# GitHub release URL base
RELEASE_URL="https://github.com/user/rcurl/releases/download/$VERSION"

# Download and compute checksums
echo "Downloading release artifacts..."

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Define platforms
declare -A PLATFORMS=(
    ["darwin-aarch64"]="rcurl-darwin-aarch64.tar.gz"
    ["darwin-x86_64"]="rcurl-darwin-x86_64.tar.gz"
    ["linux-aarch64"]="rcurl-linux-aarch64.tar.gz"
    ["linux-x86_64"]="rcurl-linux-x86_64.tar.gz"
    ["windows-x86_64"]="rcurl-windows-x86_64.zip"
)

declare -A CHECKSUMS

for platform in "${!PLATFORMS[@]}"; do
    file="${PLATFORMS[$platform]}"
    echo "  Downloading $file..."

    if curl -sL "$RELEASE_URL/$file" -o "$TEMP_DIR/$file"; then
        checksum=$(sha256sum "$TEMP_DIR/$file" | cut -d' ' -f1)
        CHECKSUMS[$platform]="$checksum"
        echo "    SHA256: $checksum"
    else
        echo "    Warning: Failed to download $file"
    fi
done

# Update Homebrew formula
echo ""
echo "Updating Homebrew formula..."

HOMEBREW_FILE="packaging/homebrew/rcurl.rb"
if [ -f "$HOMEBREW_FILE" ]; then
    sed -i "s/version \"[^\"]*\"/version \"$VERSION_NUM\"/" "$HOMEBREW_FILE"

    if [ -n "${CHECKSUMS[darwin-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_ARM64/${CHECKSUMS[darwin-aarch64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[darwin-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_X64/${CHECKSUMS[darwin-x86_64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_ARM64/${CHECKSUMS[linux-aarch64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_X64/${CHECKSUMS[linux-x86_64]}/" "$HOMEBREW_FILE"
    fi

    echo "  Updated $HOMEBREW_FILE"
fi

# Update Scoop manifest
echo "Updating Scoop manifest..."

SCOOP_FILE="packaging/scoop/rcurl.json"
if [ -f "$SCOOP_FILE" ]; then
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION_NUM\"/" "$SCOOP_FILE"
    sed -i "s|/v[0-9.]*-*[a-z0-9]*/|/$VERSION/|g" "$SCOOP_FILE"

    if [ -n "${CHECKSUMS[windows-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_WINDOWS_X64/${CHECKSUMS[windows-x86_64]}/" "$SCOOP_FILE"
        # Also update existing SHA256 hashes
        sed -i "s/\"hash\": \"[a-f0-9]*\"/\"hash\": \"${CHECKSUMS[windows-x86_64]}\"/" "$SCOOP_FILE"
    fi

    echo "  Updated $SCOOP_FILE"
fi

echo ""
echo "Done! Review the changes and commit:"
echo "  git diff packaging/"
echo "  git add packaging/"
echo "  git commit -m 'Update package manifests for $VERSION'"
