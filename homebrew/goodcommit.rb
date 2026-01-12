class Goodcommit < Formula
  desc "Good Commit: fast, reliable AI commit messages"
  homepage "https://github.com/Bikz/goodcommit"
  version "0.2.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.1/goodcommit-aarch64-apple-darwin.tar.gz"
      sha256 "a3619932cb4f4d14fd02b29cd63e4d0a33f61065534b59e3374b2ebd49089272"
    else
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.1/goodcommit-x86_64-apple-darwin.tar.gz"
      sha256 "3cff154ad011aae88e8aab751286ca5c23e38b15c8cd9f2a0a0f9e67b0f04dba"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/goodcommit/releases/download/v0.2.1/goodcommit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "a120278f3ebaeb4c7e132ab590f6bf3a907890c1149276daf5297275bb856dbb"
  end

  def install
    bin.install "goodcommit"
    bin.install_symlink "goodcommit" => "g"
    bin.install_symlink "goodcommit" => "g."
  end

  def caveats
    <<~EOS
      Next steps:
        goodcommit setup

      Commands:
        g
        g.
    EOS
  end

  test do
    assert_match "goodcommit", shell_output("#{bin}/goodcommit --help")
  end
end
