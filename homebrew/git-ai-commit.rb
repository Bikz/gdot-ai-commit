class GitAiCommit < Formula
  desc "One-command AI commit messages with GPT-5 and Ollama"
  homepage "https://github.com/Bikz/git-ai-commit"
  version "0.1.3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.3/git-ai-commit-aarch64-apple-darwin.tar.gz"
      sha256 "f4a6abf975462b724cd9cf39d3a1aecdc31a211d88ada6a3cb92891e0cb6b397"
    else
      url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.3/git-ai-commit-x86_64-apple-darwin.tar.gz"
      sha256 "30848d3d3461a39f0aa9ae4d6dca681250763943f94d8c258b54dd3de8337463"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "linux arm64 builds are not yet available"
    end

    url "https://github.com/Bikz/git-ai-commit/releases/download/v0.1.3/git-ai-commit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "9d45d7ac0be7e59ae66a15b03b1c043ecf1fc469210aa65a05ce2006e5bea046"
  end

  def install
    bin.install "git-ai-commit"
    bin.install_symlink "git-ai-commit" => "g"
    bin.install_symlink "git-ai-commit" => "g."
  end

  test do
    assert_match "git-ai-commit", shell_output("#{bin}/git-ai-commit --help")
  end
end
