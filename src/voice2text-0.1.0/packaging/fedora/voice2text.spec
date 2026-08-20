Name:           voice2text
Version:        0.1.0
Release:        1%{?dist}
Summary:        Linux voice-to-text dictation tool using Whisper

License:        MIT
URL:            https://github.com/harry1489/voice2text
Source0:        %{url}/archive/refs/tags/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rustc
BuildRequires:  cmake
BuildRequires:  clang-devel
BuildRequires:  pkgconfig(alsa)

%description
voice2text is a push-to-talk voice dictation tool for Linux. Hold a
configurable hotkey (default: F23), speak into your microphone, and your
speech is transcribed locally using OpenAI's Whisper model and typed into
the active window.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm755 install.sh %{buildroot}%{_datadir}/%{name}/install.sh
install -d %{buildroot}%{_datadir}/%{name}/models

%files
%{_bindir}/%{name}
%{_datadir}/%{name}/
%doc README.md
%license LICENSE

%changelog
* Mon Aug 18 2025 harry <harry1489@users.noreply.github.com> - 0.1.0-1
- Initial release
