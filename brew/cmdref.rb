class Cmdref < Formula
  desc "Interactive command reference tool for Linux, macOS, Windows and testing commands"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-macos-aarch64"
      sha256 "SHA256_PLACEHOLDER_MACOS_AARCH64"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-macos-x86_64"
      sha256 "SHA256_PLACEHOLDER_MACOS_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-linux-aarch64"
      sha256 "SHA256_PLACEHOLDER_LINUX_AARCH64"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-linux-x86_64"
      sha256 "SHA256_PLACEHOLDER_LINUX_X86_64"
    end
  end

  def install
    # 找到下载的二进制文件并重命名为 cmdref
    binary = Dir["*"].first
    bin.install binary => "cmdref"
  end

  test do
    assert_match "cmdref", shell_output("#{bin}/cmdref --version")
  end
end
