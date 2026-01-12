class Goodcommit < Formula
  desc "Good Commit: fast, reliable AI commit messages"
  homepage "https://github.com/Bikz/goodcommit"
  version "0.2.3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.3/goodcommit-aarch64-apple-darwin.tar.gz"
      sha256 "fe17304767c15dc7bcff473794f88ef374fd40ae879e142274d39741c8858e55"
    else
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.3/goodcommit-x86_64-apple-darwin.tar.gz"
      sha256 "5b4b99ca5929f1341bd187fddf22ab2642fcd7deab73ac3341d37df8634f5f1b"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/goodcommit/releases/download/v0.2.3/goodcommit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "c883f05ddcc336bb0bed042da2aea2559f4c1c46e4dd4ab258266cf1bb4659cc"
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
