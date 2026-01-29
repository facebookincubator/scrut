class Scrut < Formula
  desc "Simple and powerful test framework for CLI applications"
  homepage "https://facebookincubator.github.io/scrut/"
  version "0.4.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/facebookincubator/scrut/releases/download/v#{version}/scrut-v#{version}-macos-aarch64.tar.gz"
      sha256 "b05bf41457af26c7ed9e0dd6434c876e84b2904539dc7c2ffd9c8527fd4883c1"
    elsif Hardware::CPU.intel?
      url "https://github.com/facebookincubator/scrut/releases/download/v#{version}/scrut-v#{version}-macos-x86_64.tar.gz"
      sha256 "652ee38e11a6e9da7e035d5d95b796c05e338214186a8587b73440ef467dbdcf"
    else
      odie "scrut: unsupported macOS architecture: #{Hardware::CPU.arch}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/facebookincubator/scrut/releases/download/v#{version}/scrut-v#{version}-linux-aarch64.tar.gz"
      sha256 "5373cc51f85f9b24847a74412b66c4524e987c5b295799b3a98661e7a9f1e944"
    elsif Hardware::CPU.intel?
      url "https://github.com/facebookincubator/scrut/releases/download/v#{version}/scrut-v#{version}-linux-x86_64.tar.gz"
      sha256 "894b12768b5886b8ad244fd6fad19f334c820f296d9f96befacdff32b90b1e6b"
    else
      odie "scrut: unsupported Linux architecture: #{Hardware::CPU.arch}"
    end
  end

  def install
    bin.install "scrut"

    generate_completions_from_executable(bin/"scrut", shells: [:bash, :fish, :pwsh, :zsh]) do |shell|
      env_value = { bash: "bash_source", fish: "fish_source", pwsh: "powershell_source", zsh: "zsh_source" }.fetch(shell)
      Utils.safe_popen_read({ "_SCRUT_COMPLETE" => env_value }, bin/"scrut")
    end
  end

  test do
    assert_match "scrut #{version}", shell_output("#{bin}/scrut --version")
  end
end
