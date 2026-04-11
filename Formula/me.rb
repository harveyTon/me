class Me < Formula
  desc "A modern, context-aware replacement for whoami"
  homepage "https://github.com/harveyTon/me"
  url "https://github.com/harveyTon/me/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "fe63f5e6fa5e866a42a2c966e5fde142ac36bc8f39aeb37de8e1406527f64594"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    man1.install "man/man1/me.1"
  end
end
