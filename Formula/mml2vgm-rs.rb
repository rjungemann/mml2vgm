class Mml2vgmRs < Formula
  desc "Music Macro Language to VGM/XGM/ZGM compiler for retro game sound chips"
  homepage "https://github.com/rjungemann/maltese"
  license "GPL-3.0-only"

  head "https://github.com/rjungemann/maltese.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "mml2vgm-rs")

    # Shell completions
    bash_completion.install "mml2vgm-rs/completions/mml2vgm-rs.bash"
    zsh_completion.install "mml2vgm-rs/completions/_mml2vgm-rs"

    # Man page
    man1.install "mml2vgm-rs/docs/mml2vgm-rs.1"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mml2vgm-rs --version")

    (testpath/"smoke.gwi").write <<~GWI
      '{
          TitleName  = Homebrew Smoke Test
          Format     = VGM
          ClockCount = 192
          Octave-Rev = FALSE
          PartYM2612 = A
      }
      '@ M 000
         AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
      '@ 031,000,000,007,000,000,000,001,000,000,000
      '@ 031,000,000,007,000,000,000,001,000,000,000
      '@ 031,000,000,007,000,000,000,001,000,000,000
      '@ 031,000,000,007,000,000,042,000,001,000,000
         AL  FB
      '@ 004,000
      'A1 t120 @0 v100 l4 o4 c e g >c
    GWI

    system bin/"mml2vgm-rs", "--check", testpath/"smoke.gwi"
  end
end
