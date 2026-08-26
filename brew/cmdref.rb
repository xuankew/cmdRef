class Cmdref < Formula
  desc "Interactive command reference tool — 189 commands across 5 platforms"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.3.0/cmdref-macos-aarch64"
      sha256 "7879caca02117be8e5122164d9edc3fed22446613062a9b4b076c8fa99fcaa47"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.3.0/cmdref-macos-x86_64"
      sha256 "42de0fbe568ecdf81c7ad86772eaa980a75febed61baab13830952aaf0384e4c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.3.0/cmdref-linux-aarch64"
      sha256 "b5f15c74fc95603a4db1c9271068497c4b341288b6024f9a1689ef7c873aa54b"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.3.0/cmdref-linux-x86_64"
      sha256 "40199bff110f4937683fa89bc41ed1b6da484eecc22cf8fc3eca48eabaf36566"
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
