class Goodcommit < Formula
  desc "Good Commit: fast, reliable AI commit messages"
  homepage "https://github.com/Bikz/goodcommit"
  version "0.2.4"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.4/goodcommit-aarch64-apple-darwin.tar.gz"
      sha256 "e447262acececf4bf37e9f8722fa9d7f9f11fadbe790b634b20320c0f926632c"
    else
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.4/goodcommit-x86_64-apple-darwin.tar.gz"
      sha256 "7e755accd9081a1922109492fe256f35bc94971f1718402fbb59f1a346a933d6"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/goodcommit/releases/download/v0.2.4/goodcommit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "d2bc2183577f944b8abd1791ea2f1e2750a60e67a3ebb59cdb73e3328e422c79"
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
