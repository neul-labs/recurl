# Homebrew formula for rcurl
# Install with: brew install rcurl/tap/rcurl
# Or tap first: brew tap rcurl/tap && brew install rcurl

class Rcurl < Formula
  desc "Drop-in curl replacement with automatic anti-bot bypass"
  homepage "https://github.com/user/rcurl"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/user/rcurl/releases/download/v#{version}/rcurl-darwin-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_ARM64"
    end
    on_intel do
      url "https://github.com/user/rcurl/releases/download/v#{version}/rcurl-darwin-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/user/rcurl/releases/download/v#{version}/rcurl-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/user/rcurl/releases/download/v#{version}/rcurl-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  def install
    bin.install "bin/rcurl"
    bin.install "bin/rcurld" if File.exist?("bin/rcurld")
  end

  def caveats
    <<~EOS
      rcurl is installed! To use it as a drop-in curl replacement, add an alias:

        echo 'alias curl="rcurl"' >> ~/.zshrc   # or ~/.bashrc

      Or use rcurl directly:

        rcurl https://example.com

      For JS preflight with warm browsers, start the daemon:

        rcurld start

    EOS
  end

  test do
    system "#{bin}/rcurl", "--version"
    system "#{bin}/rcurl", "-s", "https://httpbin.org/get"
  end
end
