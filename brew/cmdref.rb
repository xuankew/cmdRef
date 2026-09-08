class Cmdref < Formula
  desc "Interactive command reference tool for Linux, macOS, Windows and testing commands"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.6.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.6.0/cmdref-macos-aarch64"
      sha256 "83c6998370f90a0294e1e50e18357f25038718dee60f7e1625b30e8d0b53e1d4"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.6.0/cmdref-macos-x86_64"
      sha256 "2a83265a6d805ed16dc2b20c7375045742eb30de4fa11ae4a2849b3b02446405"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.6.0/cmdref-linux-aarch64"
      sha256 "8f7b7b3405f1dbf9e1c584cb78f183b07aefbcece97d6b0d5f5b43d0d4414a17"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.6.0/cmdref-linux-x86_64"
      sha256 "65936814f67695d99f5594866856247a8d55968327bdf18e39f87e0f8d79bf2b"
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
