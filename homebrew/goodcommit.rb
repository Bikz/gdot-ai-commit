class Goodcommit < Formula
  desc "Good Commit: fast, reliable AI commit messages"
  homepage "https://github.com/Bikz/goodcommit"
  version "0.2.2"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.2/goodcommit-aarch64-apple-darwin.tar.gz"
      sha256 "a27045276a4f2a8745d8cce5b27acab17564c151ecd9af56b03509a38674e911"
    else
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.2/goodcommit-x86_64-apple-darwin.tar.gz"
      sha256 "62286bda84b90e16a475692f54f2e830306210eaad3fffa69baab42c740daeef"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/goodcommit/releases/download/v0.2.2/goodcommit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "f9960b4bed7f5b9646d3e638e81d0469df5a7e2f9e66ecb0ae40da465ab85322"
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
