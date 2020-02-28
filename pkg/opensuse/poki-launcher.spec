#
# spec file for package poki-launcher
#
# Copyright (c) 2019 Ben Aaron Goldberg <benaagoldberg@gmail.com>
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

# Please submit bugfixes or comments via https://bugs.opensuse.org/
#


Name:           poki-launcher
Version:        0.5.0
Release:        1
Summary:        An application launcher for Linux
License:        GPL-3.0
URL:            https://git.sr.ht/~zethra/poki-launcher
Source0:        https://git.sr.ht/~zethra/poki-launcher/archive/%{version}.tar.gz
Source1:        vendor.tar.xz
Source2:        cargo-config
BuildRequires:  rust >= 1.34
BuildRequires:  cargo rust gcc
%if 0%{?suse_version}
BuildRequires:  libqt5-qtbase-devel libqt5-qtdeclarative-devel
%endif
%if 0%{?fedora}
BuildRequires:  qt5-qtbase-devel qt5-qtdeclarative-devel
%endif

%description
An application launcher for Linux

%prep
%setup -q
%setup -q -D -T -a 1
mkdir .cargo
cp %{SOURCE2} .cargo/config
mkdir bin
ln -s /usr/bin/qmake-qt5 ./bin/qmake

%build
export PATH=$PATH:$PWD/bin
%if 0%{?fedora}
export CFLAGS="${RPM_OPT_FLAGS}"
export CXXFLAGS="${RPM_OPT_FLAGS}"
%endif
rustc -V
cargo build --release

%install
install -Dm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm 0444 %{name}.hjson %{buildroot}%{_datarootdir}/defaults/%{name}.hjson
rm -f debugsourcefiles.list

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_datarootdir}/defaults
%{_datarootdir}/defaults/%{name}.hjson

%changelog
