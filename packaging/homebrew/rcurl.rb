# Homebrew formula for recurl
# Install with: brew install recurl/tap/recurl
# Or tap first: brew tap recurl/tap && brew install recurl

class Recurl < Formula
  desc "Drop-in curl replacement with automatic anti-bot bypass"
  homepage "https://github.com/user/recurl"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/user/recurl/releases/download/v#{version}/recurl-darwin-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_ARM64"
    end
    on_intel do
      url "https://github.com/user/recurl/releases/download/v#{version}/recurl-darwin-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/user/recurl/releases/download/v#{version}/recurl-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/user/recurl/releases/download/v#{version}/recurl-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  def install
    bin.install "bin/recurl"
    bin.install "bin/recurld" if File.exist?("bin/recurld")
  end

  def caveats
    <<~EOS
      recurl is installed! To use it as a drop-in curl replacement, add an alias:

        echo 'alias curl="recurl"' >> ~/.zshrc   # or ~/.bashrc

      Or use recurl directly:

        recurl https://example.com

      For JS preflight with warm browsers, start the daemon:

        recurld start

    EOS
  end

  test do
    system "#{bin}/recurl", "--version"
    system "#{bin}/recurl", "-s", "https://httpbin.org/get"
  end
end
