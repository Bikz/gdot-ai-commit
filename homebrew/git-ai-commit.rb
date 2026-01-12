class GitAiCommit < Formula
  desc "One-command AI commit messages with GPT-5 and Ollama"
  homepage "https://github.com/Bikz/git-ai-commit"
  version "0.1.4"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.4/git-ai-commit-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    else
      url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.4/git-ai-commit-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.4/git-ai-commit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_ME"
  end

  def install
    bin.install "git-ai-commit"
    bin.install_symlink "git-ai-commit" => "g"
    bin.install_symlink "git-ai-commit" => "g."
  end

  def caveats
    <<~EOS
      Next steps:
        git-ai-commit setup

      Commands:
        g
        g.
    EOS
  end

  test do
    assert_match "git-ai-commit", shell_output("#{bin}/git-ai-commit --help")
  end
end
