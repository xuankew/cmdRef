class Cmdref < Formula
  desc "Interactive command reference tool for Linux, macOS, Windows and testing commands"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-macos-aarch64"
      sha256 "11954f93b215aa97d62a012da23c3c0be7fe4404fac82ed9d8ac09e8eb1d951b"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-macos-x86_64"
      sha256 "2be0bb707ce3aaafb05df6bd03a5471d8f1896cd2a96547178724717d0732676"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-linux-aarch64"
      sha256 "9275ad5b5ccae0f9ea3a000b0a2c4629e9d3a8201f160cb983837577d5ca980d"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.1.0/cmdref-linux-x86_64"
      sha256 "ab9f0b69b63be51188d433fb4c797f331674b757b05a30655e6e384ac9e0913e"
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
