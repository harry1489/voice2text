{ lib
, rustPlatform
, pkg-config
, cmake
, clang
, alsa-lib
, installShellFiles
}:

rustPlatform.buildRustPackage {
  pname = "voice2text";
  version = "0.1.0";

  src = ./../..;

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  nativeBuildInputs = [ pkg-config cmake clang ];

  buildInputs = [ alsa-lib ];

  postInstall = ''
    install -Dm755 ${./../../install.sh} $out/share/voice2text/install.sh
    mkdir -p $out/share/voice2text/models
  '';

  meta = with lib; {
    description = "Linux voice-to-text dictation tool using Whisper";
    homepage = "https://github.com/harry1489/voice2text";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "voice2text";
  };
}
