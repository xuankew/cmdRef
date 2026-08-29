class Cmdref < Formula
  desc "Interactive command reference tool for Linux, macOS, Windows and testing commands"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.5.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.5.0/cmdref-macos-aarch64"
      sha256 "cd65d9e6634b626f081c03f131e03ac0c53cd0e96a1f78a3117a37b1d222ee71"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.5.0/cmdref-macos-x86_64"
      sha256 "d4ce86ffdbf475c2290685ae112f8d77c9c660d08aaf3fd0fd28e714f4e02a66"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.5.0/cmdref-linux-aarch64"
      sha256 "6ef7e8f6465fad54860c7e5bfb79dd9601f7dae7f9c2937d7a8eefac5dd8302c"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.5.0/cmdref-linux-x86_64"
      sha256 "b9985da08ff55ad681766e3620fc54d7816b56e89d02bcbb2b2bb8d51f750327"
    end
  end

  def install
    binary = Dir["*"].first
    bin.install binary => "cmdref"
  end

  test do
    assert_match "cmdref", shell_output("#{bin}/cmdref --version")
  end
end
