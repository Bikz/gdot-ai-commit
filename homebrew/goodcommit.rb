class Goodcommit < Formula
  desc "Good Commit: fast, reliable AI commit messages"
  homepage "https://github.com/Bikz/goodcommit"
  version "0.2.5"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.5/goodcommit-aarch64-apple-darwin.tar.gz"
      sha256 "d16a90485b1a16f3f8905a6929ffa4417940a6629e5f58e0c058ad5256295b6b"
    else
      url "https://github.com/Bikz/goodcommit/releases/download/v0.2.5/goodcommit-x86_64-apple-darwin.tar.gz"
      sha256 "117cd261d300319e903e5bfd55726dc79f35b03e39028625e5ad3db2641bb665"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/goodcommit/releases/download/v0.2.5/goodcommit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "e856dd6e0d91cdd865d81ac7664f0808c659a8747b2239ebb13ad83b9084a794"
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
