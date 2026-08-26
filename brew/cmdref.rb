class Cmdref < Formula
  desc "Interactive command reference tool for Linux, macOS, Windows and testing commands"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.2.0/cmdref-macos-aarch64"
      sha256 "188d28d40a365c074ade1f8ed6848b0d41fda8372a312eb74a7b20e8eeb5ad81"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.2.0/cmdref-macos-x86_64"
      sha256 "8f4bc5d826482cd0567abe049ec3eb996a8a72adfe2299bf252fd13ba9f06337"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.2.0/cmdref-linux-aarch64"
      sha256 "5948446f48af66ace3ef756d33a3344459b6387d0565d21986dfea0189e234b5"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.2.0/cmdref-linux-x86_64"
      sha256 "4d5daddeca7d76eea4be69c8a4cff976b7f0a63eeee4aaca1b4274bccc87158c"
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
