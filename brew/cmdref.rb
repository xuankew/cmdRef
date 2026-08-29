class Cmdref < Formula
  desc "Interactive command reference tool — 189 commands across 5 platforms"
  homepage "https://github.com/xuankew/cmdRef"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.4.0/cmdref-macos-aarch64"
      sha256 "65371a66ecf2572c1deb309edbbdd632d7809d7cc2038c72e47206104659d841"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.4.0/cmdref-macos-x86_64"
      sha256 "693ae063ffaa9a83b26485bdae80f2fbda85f7c5094463ee454ff2e2219974f7"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/xuankew/cmdRef/releases/download/v0.4.0/cmdref-linux-aarch64"
      sha256 "bddb7f3eb2d4a73fce13d144692459057094c5d2a9fa947d9c3bd49108c3ec82"
    else
      url "https://github.com/xuankew/cmdRef/releases/download/v0.4.0/cmdref-linux-x86_64"
      sha256 "ce90716e9ed1771ef856b1f518b8b0d5733ca05f7dd60ef46d2e310b69bf1c1d"
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

