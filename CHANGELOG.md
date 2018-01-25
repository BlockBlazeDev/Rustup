# 1.9.0

* [Warn when tools are missing and allow an override][pr1337]

[pr1337]: https://github.com/rust-lang-nursery/rustup.rs/pull/1337

Contributors: Nick Cameron, Steffen Butzer

# 1.9.0

* [Fix self update errors filling in missing proxies][pr1326]

[pr1326]: https://github.com/rust-lang-nursery/rustup.rs/pull/1326

Contributors: Alex Crichton

# 1.8.0

* [Add `rustup run --install`][pr1295]
* [Prevent `rustup update` to a toolchain without `rustc` or `cargo`][pr1298]
* [Add support for `rustfmt` shims][pr1294]

[pr1295]: https://github.com/rust-lang-nursery/rustup.rs/pull/1295
[pr1298]: https://github.com/rust-lang-nursery/rustup.rs/pull/1298
[pr1294]: https://github.com/rust-lang-nursery/rustup.rs/pull/1294

Contributors: Alex Crichton, kennytm, Nick Cameron, Simon Sapin, Who? Me?!

# 1.7.0

* [Improve clarity of component errors][pr1255]
* [Support `--default-toolchain none`][pr1257]
* [Automatically install override toolchain when missing][pr1250]

[pr1255]: https://github.com/rust-lang-nursery/rustup.rs/pull/1255
[pr1257]: https://github.com/rust-lang-nursery/rustup.rs/pull/1257
[pr1250]: https://github.com/rust-lang-nursery/rustup.rs/pull/1250

Contributors: Aidan Hobson Sayers, Alan Du, Alex Crichton, Christoph Wurst,
Jason Mobarak, Leon Isenberg, Simon Sapin, Vadim Petrochenkov

# 1.6.0

* [Fix support for s390x][pr1228]
* [Fix `show` so it displays helpful information if the active toolchain is not installed][pr1189]
* [Fix uninstalling toolchains with stale symlinks][pr1201]
* [Replace the hyper backend with a reqwest downloading backend][pr1222]
* [Consistently give a toolchain argument in the help text][pr1212]
* [Use `exec` on Unix where possible to help manage Unix signals][pr1242]

[pr1228]: https://github.com/rust-lang-nursery/rustup.rs/pull/1228
[pr1189]: https://github.com/rust-lang-nursery/rustup.rs/pull/1189
[pr1201]: https://github.com/rust-lang-nursery/rustup.rs/pull/1201
[pr1222]: https://github.com/rust-lang-nursery/rustup.rs/pull/1222
[pr1212]: https://github.com/rust-lang-nursery/rustup.rs/pull/1212
[pr1242]: https://github.com/rust-lang-nursery/rustup.rs/pull/1242

Contributors: Alex Crichton, Chen Rotem Levy, Krishna Sundarram, Martin Geisler,
Matt Brubeck, Matt Ickstadt, Michael Benfield, Michael Fletcher, Nick Cameron,
Patrick Reisert, Ralf Jung, Sean McArthur, Steven Fackler

# 1.5.0

* [Rename references to multirust to rustup where applicable](https://github.com/rust-lang-nursery/rustup.rs/pull/1148)
* [Update platform support in README](https://github.com/rust-lang-nursery/rustup.rs/pull/1159)
* [Allow rustup to handle unavailable packages](https://github.com/rust-lang-nursery/rustup.rs/pull/1063)
* [Update libz-sys and curl-sys](https://github.com/rust-lang-nursery/rustup.rs/pull/1176)
* [Teach rustup to override the toolchain from a version file](https://github.com/rust-lang-nursery/rustup.rs/pull/1172)
* [Update sha2 crate](https://github.com/rust-lang-nursery/rustup.rs/pull/1162)
* [Check for unexpected cargo/rustc before install](https://github.com/rust-lang-nursery/rustup.rs/pull/705)
* [Update PATH in .bash_profile](https://github.com/rust-lang-nursery/rustup.rs/pull/1179)

Contributors: Allen Welkie, bors, Brian Anderson, Diggory Blake, Erick
Tryzelaar, Ricardo Martins, Артём Павлов [Artyom Pavlov]

# 1.4.0

* [set_file_perms: if the file is already executable, keep it executable](https://github.com/rust-lang-nursery/rustup.rs/pull/1141)
* [Disable man support on Windows](https://github.com/rust-lang-nursery/rustup.rs/pull/1139)
* [VS 2017 updates](https://github.com/rust-lang-nursery/rustup.rs/pull/1145)
* [Show version of rust being installed](https://github.com/rust-lang-nursery/rustup.rs/pull/1025)
* [Detect MSVC 2017](https://github.com/rust-lang-nursery/rustup.rs/pull/1136)
* [Use same precision as rustc for commit sha](https://github.com/rust-lang-nursery/rustup.rs/pull/1134)
* [Fix prompt asking for msvc even though -y is provided](https://github.com/rust-lang-nursery/rustup.rs/pull/1124)
* [README: fix rust build dir](https://github.com/rust-lang-nursery/rustup.rs/pull/1135)
* [Add support for XZ-compressed packages](https://github.com/rust-lang-nursery/rustup.rs/pull/1100)
* [Add PATH in post-install message when not modifying PATH](https://github.com/rust-lang-nursery/rustup.rs/pull/1126)
* [Cleanup download-related code in the rustup_dist crate](https://github.com/rust-lang-nursery/rustup.rs/pull/1131)
* [Increase Rust detection timeout to 3 seconds](https://github.com/rust-lang-nursery/rustup.rs/pull/1130)
* [Supress confusing NotADirectory error and show override missing](https://github.com/rust-lang-nursery/rustup.rs/pull/1128)
* [Don't try to update archive toolchains](https://github.com/rust-lang-nursery/rustup.rs/pull/1121)
* [Exit successfully on "update not yet available"](https://github.com/rust-lang-nursery/rustup.rs/pull/1120)
* [Add a message when removing a component](https://github.com/rust-lang-nursery/rustup.rs/pull/1119)
* [Use ShellExecute rather than start.exe to open docs on windows](https://github.com/rust-lang-nursery/rustup.rs/pull/1117)
* [Clarify that rustup update updates rustup itself](https://github.com/rust-lang-nursery/rustup.rs/pull/1113)
* [Ensure that intermediate directories exist when unpacking an entry](https://github.com/rust-lang-nursery/rustup.rs/pull/1098)
* [Add the rust lib dir (containing std-<hash>.dll) to the path on windows](https://github.com/rust-lang-nursery/rustup.rs/pull/1093)
* [Add x86_64-linux-android target](https://github.com/rust-lang-nursery/rustup.rs/pull/1086)
* [Fix for help.rs suggestion](https://github.com/rust-lang-nursery/rustup.rs/pull/1107)
* [Ignore remove_override_nonexistent on windows](https://github.com/rust-lang-nursery/rustup.rs/pull/1105)
* [Update proxy setting docs](https://github.com/rust-lang-nursery/rustup.rs/pull/1088)
* [Add sensible-browser to the browser list](https://github.com/rust-lang-nursery/rustup.rs/pull/1087)
* [Added help for `rustup toolchain link`](https://github.com/rust-lang-nursery/rustup.rs/pull/1017)

Contributors: Andrea Canciani, bors, Brian Anderson, CrazyMerlyn, Diggory Blake,
Fabio B, James Elford, Jim McGrath, johnthagen, Josh Lee, Kim Christensen, Marco
A L Barbosa, Mateusz Mikula, Matthew, Matt Ickstadt, Mikhail Modin, Patrick
Deuster, pxdeu, Ralf Jung, Raphaël Huchet, Robert Vally, theindigamer, Tommy Ip,
Xidorn Quan

# 1.3.0

* [Add armv8l support](https://github.com/rust-lang-nursery/rustup.rs/pull/1055)
* [Update curl crate](https://github.com/rust-lang-nursery/rustup.rs/pull/1101)
* [Fix inadvertent dependency on bash](https://github.com/rust-lang-nursery/rustup.rs/pull/1048)
* [Update openssl-probe to 0.1.1](https://github.com/rust-lang-nursery/rustup.rs/pull/1061)
* [zsh completions cleanup](https://github.com/rust-lang-nursery/rustup.rs/pull/1068)
* [Alias 'rustup toolchain uninstall' to 'rustup uninstall'](https://github.com/rust-lang-nursery/rustup.rs/pull/1073)
* [Fix a typo in PowerShell completion script help](https://github.com/rust-lang-nursery/rustup.rs/pull/1076)
* [Enforce timeouts for reading rustc version](https://github.com/rust-lang-nursery/rustup.rs/pull/1071)
* [Fix OpenSSL linkage by using the final install-directory in the build](https://github.com/rust-lang-nursery/rustup.rs/pull/1065)

Contributors: bors, Brian Anderson, Diggory Blake, Greg Alexander, James Elford,
Jordan Hiltunen, Justin Noah, Kang Seonghoon, Kevin K, Marco A L Barbosa

# 1.2.0

* [Check ZDOTDIR when adding path to .zprofile](https://github.com/rust-lang-nursery/rustup.rs/pull/1038)
* [Update links and install page to include android support](https://github.com/rust-lang-nursery/rustup.rs/pull/1037)
* [Add bash completion guidance for macOS users](https://github.com/rust-lang-nursery/rustup.rs/pull/1035)
* [Support partial downloads](https://github.com/rust-lang-nursery/rustup.rs/pull/1020)
* [Don't crash if modifying multiple profile files](https://github.com/rust-lang-nursery/rustup.rs/pull/1040)

Contributors: Brian Anderson, James Elford, Jason Dreyzehner, Marco A
L Barbosa, Wim Looman

# 1.1.0

* [Fix browser detection for Linux ppc64 and NetBSD](https://github.com/rust-lang-nursery/rustup.rs/pull/875)
* [Update windows info](https://github.com/rust-lang-nursery/rustup.rs/pull/879)
* [Update to markdown 0.2](https://github.com/rust-lang-nursery/rustup.rs/pull/896)
* [Make running program extension case insensitive](https://github.com/rust-lang-nursery/rustup.rs/pull/887)
* [Add MIPS/s390x builders (with PPC64 compilation fixed)](https://github.com/rust-lang-nursery/rustup.rs/pull/890)
* [Fix two missing quotes of download error message](https://github.com/rust-lang-nursery/rustup.rs/pull/867)
* [www: MIPS support and cleanups](https://github.com/rust-lang-nursery/rustup.rs/pull/866)
* [Update release instructions](https://github.com/rust-lang-nursery/rustup.rs/pull/863)
* [Don't set low speed limits for curl](https://github.com/rust-lang-nursery/rustup.rs/pull/914)
* [Attempt to fix msi build. Pin appveyor nightlies](https://github.com/rust-lang-nursery/rustup.rs/pull/910)
* [Stop defaulting to $PATH searches when the binary can't be found and causing infinite recursion](https://github.com/rust-lang-nursery/rustup.rs/pull/917)
* [Upgrade openssl](https://github.com/rust-lang-nursery/rustup.rs/pull/934)
* [Improve browser detection and install instructions](https://github.com/rust-lang-nursery/rustup.rs/pull/936)
* [Add android support to rustup-init.sh](https://github.com/rust-lang-nursery/rustup.rs/pull/949)
* [Add fallback to symlink if hardlink fails](https://github.com/rust-lang-nursery/rustup.rs/pull/951)
* [readme: add tmp dir hint to Contributing section](https://github.com/rust-lang-nursery/rustup.rs/pull/985)
* [Fixed link to the list of supported platforms](https://github.com/rust-lang-nursery/rustup.rs/pull/970)
* [Update job object code to match Cargo's](https://github.com/rust-lang-nursery/rustup.rs/pull/984)
* [Added argument-documentation to rustup-init.sh](https://github.com/rust-lang-nursery/rustup.rs/pull/962)
* [Add/remove multiple toolchains](https://github.com/rust-lang-nursery/rustup.rs/pull/986)
* [Remove curl usage from appveyor](https://github.com/rust-lang-nursery/rustup.rs/pull/1001)
* [Store downloaded files in a persistent directory until installation](https://github.com/rust-lang-nursery/rustup.rs/pull/958)
* [Add android build support](https://github.com/rust-lang-nursery/rustup.rs/pull/1000)
* [Fix up a bunch of things indicated by clippy](https://github.com/rust-lang-nursery/rustup.rs/pull/1012)
* [Ensure librssl compatibility](https://github.com/rust-lang-nursery/rustup.rs/pull/1011)
* [RLS support](https://github.com/rust-lang-nursery/rustup.rs/pull/1005)
* [Add 'docs' alias](https://github.com/rust-lang-nursery/rustup.rs/pull/1010)
* [Use correct name for undefined linked toolchain invocation](https://github.com/rust-lang-nursery/rustup.rs/pull/1008)
* [zsh install support](https://github.com/rust-lang-nursery/rustup.rs/pull/1013)
* [Add/remove multiple components+targets](https://github.com/rust-lang-nursery/rustup.rs/pull/1016)
* [Better error message when not running in a tty](https://github.com/rust-lang-nursery/rustup.rs/pull/1026)
* [Indent help text](https://github.com/rust-lang-nursery/rustup.rs/pull/1019)
* [Document installing to a custom location using CARGO_HOME and RUSTUP_HOME environment variables](https://github.com/rust-lang-nursery/rustup.rs/pull/1024)
* [Aggressive remove_dir_all](https://github.com/rust-lang-nursery/rustup.rs/pull/1015)

Contributors: Aarthi Janakiraman, Alex Burka, Alex Crichton, bors,
Brian Anderson, Christian Muirhead, Christopher Armstrong, Daniel
Lockyer, Diggory Blake, Evgenii Pashkin, Grissiom, James Elford, Luca
Bruno, Lyuha, Manish Goregaokar, Marc-Antoine Perennou, Marco A L
Barbosa, Mikhail Pak, Nick Cameron, polonez, Sam Marshall, Steve
Klabnik, Tomáš Hübelbauer, topecongiro, Wang Xuerui

# 1.0.0

* [Statically link MSVC CRT](https://github.com/rust-lang-nursery/rustup.rs/pull/843)
* [Upgrade ~/.multirust correctly from rustup-init](https://github.com/rust-lang-nursery/rustup.rs/pull/858)

Contributors: Alex Crichton, Andrew Koroluk, Arch, benaryorg, Benedikt Reinartz,
Björn Steinbrink, bors, Boutin, Michael, Brian Anderson, Cam Swords, Chungmin
Park, Corey Farwell, Daniel Keep, David Salter, Diggory Blake, Drew Fisher,
Erick Tryzelaar, Florian Gilcher, geemili, Guillaume Fraux, Ivan Nejgebauer,
Ivan Petkov, Jacob Shaffer, Jake Goldsborough, James Lucas, Jeremiah Peschka,
jethrogb, Jian Zeng, Jimmy Cuadra, Joe Wilm, Jorge Aparicio, Josh Machol, Josh
Stone, Julien Blanchard, Kai Noda, Kai Roßwag, Kamal Marhubi, Kevin K, Kevin
Rauwolf, Kevin Yap, Knight, leonardo.yvens, llogiq, Marco A L Barbosa, Martin
Pool, Matt Brubeck, mdinger, Michael DeWitt, Mika Attila, Nate Mara, NODA, Kai,
Oliver Schneider, Patrick Reisert, Paul Padier, Ralph Giles, Raphael Cohn, Ri,
Ricardo Martins, Ryan Havar, Ryan Kung, Severen Redwood, Tad Hardesty, Taylor
Cramer, theindigamer, Tim Neumann, Tobias Bucher, trolleyman, Vadim
Petrochenkov, Virgile Andreani, V Jackson, Vladimir, Wang Xuerui, Wayne Warren,
Wesley Moore, Yasushi Abe, Y. T. Chung

# 0.7.0

* [Correctly "detect" host endianness on MIPS](https://github.com/rust-lang-nursery/rustup.rs/pull/802)
* [Add powershell completions](https://github.com/rust-lang-nursery/rustup.rs/pull/801)
* [Update toolchain used to build rustup](https://github.com/rust-lang-nursery/rustup.rs/pull/741)
* [Support probing MIPS64 n64 targets](https://github.com/rust-lang-nursery/rustup.rs/pull/815)
* [Support MIPS architectures in rustup-init.sh](https://github.com/rust-lang-nursery/rustup.rs/pull/825)
* [Automatically detect NetBSD during standard install](https://github.com/rust-lang-nursery/rustup.rs/pull/824)
* [Fix symlink creation on windows](https://github.com/rust-lang-nursery/rustup.rs/pull/823)
* [Search PATH for binaries run by `rustup run`](https://github.com/rust-lang-nursery/rustup.rs/pull/822)
* [Recursive tool invocations should invoke the proxy, not the tool directly](https://github.com/rust-lang-nursery/rustup.rs/pull/812)
* [Upgrade error-chain](https://github.com/rust-lang-nursery/rustup.rs/pull/841)
* [Add FAQ entry for downloading Rust source](https://github.com/rust-lang-nursery/rustup.rs/pull/840)
* [Rename ~/.multirust to ~/.rustup](https://github.com/rust-lang-nursery/rustup.rs/pull/830)
* [Remove some codegen hacks](https://github.com/rust-lang-nursery/rustup.rs/pull/850)
* [Update libc for MIPS64 host builds](https://github.com/rust-lang-nursery/rustup.rs/pull/847)
* [Default to MSVC on Windows](https://github.com/rust-lang-nursery/rustup.rs/pull/842)

Contributors: Alex Crichton, Arch, bors, Brian Anderson, Diggory Blake, Kai
Roßwag, Kevin K, Oliver Schneider, Ryan Havar, Tobias Bucher, Wang Xuerui

# 0.6.5

* [Update bundled curl code](https://github.com/rust-lang-nursery/rustup.rs/pull/790)
* [Remove old zsh completions](https://github.com/rust-lang-nursery/rustup.rs/pull/779)
* [Fix two small typos in the error descriptions](https://github.com/rust-lang-nursery/rustup.rs/pull/788)
* [Update README](https://github.com/rust-lang-nursery/rustup.rs/pull/782)
* [Fix name of bash completion directory](https://github.com/rust-lang-nursery/rustup.rs/pull/780)

Contributors: Alex Crichton, Björn Steinbrink, Brian Anderson, Jian Zeng, Matt
Brubeck

# 0.6.4

* [making rustup prepend cargo bin to path instead of append](https://github.com/rust-lang-nursery/rustup.rs/pull/707)
* [Use released version of rustls dependency](https://github.com/rust-lang-nursery/rustup.rs/pull/711)
* [Update OpenSSL](https://github.com/rust-lang-nursery/rustup.rs/pull/733)
* [Made outputting of ANSI terminal escapes codes defensive](https://github.com/rust-lang-nursery/rustup.rs/pull/725)
* [Adjusted rustup-init.sh need_cmd to add uname and remove printf](https://github.com/rust-lang-nursery/rustup.rs/pull/723)
* [Update to error-chain 0.5.0 to allow optional backtrace](https://github.com/rust-lang-nursery/rustup.rs/pull/591)
* [Fix variable naming in rustup-init.sh](https://github.com/rust-lang-nursery/rustup.rs/pull/737)
* [Update clap to fix --help formatting](https://github.com/rust-lang-nursery/rustup.rs/pull/738)
* [Add an FAQ entry about troubles with antivirus](https://github.com/rust-lang-nursery/rustup.rs/pull/739)
* [Clarify how rustup toolchain installation works on Windows](https://github.com/rust-lang-nursery/rustup.rs/pull/744)
* [Do not interpret commas when using "rustup run"](https://github.com/rust-lang-nursery/rustup.rs/pull/752)
* [Fix local declarations for zsh completions](https://github.com/rust-lang-nursery/rustup.rs/pull/753)
* [Fix checksum failures](https://github.com/rust-lang-nursery/rustup.rs/pull/759)
* [Treat an empty `CARGO_HOME` the same as an unset `CARGO_HOME`](https://github.com/rust-lang-nursery/rustup.rs/pull/767)
* [Check stdout is a tty before using terminal features](https://github.com/rust-lang-nursery/rustup.rs/pull/772)
* [Add completion generation for zsh, bash and fish shells](https://github.com/rust-lang-nursery/rustup.rs/pull/773)

Contributors: Alex Crichton, Andrew Koroluk, Brian Anderson, Chungmin Park,
Diggory Blake, Guillaume Fraux, Jake Goldsborough, jethrogb, Kamal Marhubi,
Kevin K, Kevin Rauwolf, Raphael Cohn, Ricardo Martins

# 0.6.3

* [Disable anti-sudo check](https://github.com/rust-lang-nursery/rustup.rs/pull/698)
* [Fixed CI toolchain pinning](https://github.com/rust-lang-nursery/rustup.rs/pull/696)

Contributors: Brian Anderson

# 0.6.2

* [Add basic autocompletion for Zsh](https://github.com/rust-lang-nursery/rustup.rs/pull/689)
* [Sort toolchains by semantic version](https://github.com/rust-lang-nursery/rustup.rs/pull/688)

Contributors: Brian Anderson, Diggory Blake, Knight, Marco A L Barbosa

# 0.6.1

* [Fix mysterious crash on OS X 10.10+](https://github.com/rust-lang-nursery/rustup.rs/pull/684)
* [Fix `component remove` command and add a test for it](https://github.com/rust-lang-nursery/rustup.rs/pull/683)

Contributors: Brian Anderson, Diggory Blake

# 0.6.0

* [Print rustup version after update](https://github.com/rust-lang-nursery/rustup.rs/pull/614)
* [Don't spawn processes for copying](https://github.com/rust-lang-nursery/rustup.rs/pull/630)
* [Upgrade error-chain to 0.3](https://github.com/rust-lang-nursery/rustup.rs/pull/636)
* [Support telemetry with lots of output](https://github.com/rust-lang-nursery/rustup.rs/pull/645)
* [Remove empty directories after component uninstall](https://github.com/rust-lang-nursery/rustup.rs/pull/634)
* [Update rustup-init.sh for powerpc](https://github.com/rust-lang-nursery/rustup.rs/pull/647)
* [Switch builds to current nightly toolchain](https://github.com/rust-lang-nursery/rustup.rs/pull/651)
* [Add a WIP MSI installer](https://github.com/rust-lang-nursery/rustup.rs/pull/635)
* [Add `--path` and `--nonexistent` options to `rustup override unset`](https://github.com/rust-lang-nursery/rustup.rs/pull/650)
* [Add `component` subcommand](https://github.com/rust-lang-nursery/rustup.rs/pull/659)

Contributors: Alex Crichton, Brian Anderson, Diggory Blake, Ivan Nejgebauer Josh
Machol, Julien Blanchard, Patrick Reisert, Ri, Tim Neumann

# 0.5.0

* [List custom toolchains in `rustup show`](https://github.com/rust-lang-nursery/rustup.rs/pull/620)
* [Add a usage example for local builds](https://github.com/rust-lang-nursery/rustup.rs/pull/622)
* [Read/Write impl rework for rustls](https://github.com/rust-lang-nursery/rustup.rs/pull/592)
* [Introduce `+TOOLCHAIN` syntax for proxies](https://github.com/rust-lang-nursery/rustup.rs/pull/615)
* [Add `rustup man`](https://github.com/rust-lang-nursery/rustup.rs/pull/616)
* [Try detecting sudo when running `rustup-init`](https://github.com/rust-lang-nursery/rustup.rs/pull/617)
* [Handle active custom toolchain in `rustup show`](https://github.com/rust-lang-nursery/rustup.rs/pull/621)

Contributors: Brian Anderson, Cam Swords, Daniel Keep, Diggory Blake,
Florian Gilcher, Ivan Nejgebauer, theindigamer

# 0.4.0

* [Improve rustls CA certificate loading](https://github.com/rust-lang-nursery/rustup.rs/pull/585)
* [Detect ARMv7 CPUs without NEON extensions and treat as ARMv6](https://github.com/rust-lang-nursery/rustup.rs/pull/593)
* [Allow any toolchain to be specified as the default during rustup installation](https://github.com/rust-lang-nursery/rustup.rs/pull/586)
* [Add details about updating rustup to README](https://github.com/rust-lang-nursery/rustup.rs/pull/590)
* [Update libbacktrace to generate less filesystem thrashing on Windows](https://github.com/rust-lang-nursery/rustup.rs/pull/604)
* [Update gcc dep to fix building on MSVC](https://github.com/rust-lang-nursery/rustup.rs/pull/605)
* [Remove the multirust binary](https://github.com/rust-lang-nursery/rustup.rs/pull/606)
* [Use the env_proxy crate for proxy environment variable handling](https://github.com/rust-lang-nursery/rustup.rs/pull/598)
* [Set system-specific dynamic loader env var for command execution](https://github.com/rust-lang-nursery/rustup.rs/pull/600)
* [Hide telemetry command from top level help](https://github.com/rust-lang-nursery/rustup.rs/pull/601)
* [Add the "no-self-update" feature](https://github.com/rust-lang-nursery/rustup.rs/pull/602)
* [Update to error-chain 0.2.2](https://github.com/rust-lang-nursery/rustup.rs/pull/609)
* [Add HTTP proxy documentation to README](https://github.com/rust-lang-nursery/rustup.rs/pull/610)

Contributors: Alex Crichton, Brian Anderson, Ivan Nejgebauer, Jimmy
Cuadra, Martin Pool, Wesley Moore

# 0.3.0

* [Teach rustup to download manifests from the `/staging/` directory](https://github.com/rust-lang-nursery/rustup.rs/pull/579).
* [Treat all HTTP client errors the same](https://github.com/rust-lang-nursery/rustup.rs/pull/578).
* [Remove winapi replacement](https://github.com/rust-lang-nursery/rustup.rs/pull/577).
* [Remove toolchain directory if initial toolchain install fails](https://github.com/rust-lang-nursery/rustup.rs/pull/574).
* [Fallback to old download methods if server returns 403](https://github.com/rust-lang-nursery/rustup.rs/pull/573).
* [Add preliminary rustls support](https://github.com/rust-lang-nursery/rustup.rs/pull/572).
* [Add a hack to remediate checksum failure issues](https://github.com/rust-lang-nursery/rustup.rs/pull/562).
* [Move error-chain out of tree](https://github.com/rust-lang-nursery/rustup.rs/pull/564).
* [Remove uses of subcommand synonyms in the examples](https://github.com/rust-lang-nursery/rustup.rs/pull/560).
* [Add `--yes` as alias for `-y`](https://github.com/rust-lang-nursery/rustup.rs/pull/563).
* [Remove unavailable toolchains from `target list`](https://github.com/rust-lang-nursery/rustup.rs/pull/553).
* [Add powerpc builds](https://github.com/rust-lang-nursery/rustup.rs/pull/534).
* [Fix help text for `rustup update`](https://github.com/rust-lang-nursery/rustup.rs/pull/552).
* [Remove noisy "rustup is up to date" message](https://github.com/rust-lang-nursery/rustup.rs/pull/550).
* [Fix references to non-existent `.rustup` directory](https://github.com/rust-lang-nursery/rustup.rs/pull/545).
* [When listing toolchains only list directories](https://github.com/rust-lang-nursery/rustup.rs/pull/544).
* [rustup-init: remove dependency on `file` command](https://github.com/rust-lang-nursery/rustup.rs/pull/543).
* [Link to rustup-init.sh in README](https://github.com/rust-lang-nursery/rustup.rs/pull/541).
* [Improve docs for `set default-host`](https://github.com/rust-lang-nursery/rustup.rs/pull/540).

Contributors: Alex Crichton, Brian Anderson, Drew Fisher, geemili,
Ivan Petkov, James Lucas, jethrogb, Kevin Yap, leonardo.yvens, Michael
DeWitt, Nate Mara, Virgile Andreani

# 0.2.0

* [Indicate correct path to remove in multirust upgrade instructions](https://github.com/rust-lang-nursery/rustup.rs/pull/518).
* [Bring back optional hyper with proxy support](https://github.com/rust-lang-nursery/rustup.rs/pull/532).
* ['default' and 'update' heuristics for bare triples](https://github.com/rust-lang-nursery/rustup.rs/pull/516).
* [Change upstream via $RUSTUP_DIST_SERVER](https://github.com/rust-lang-nursery/rustup.rs/pull/521).
* [Fail with a nicer error message if /tmp is mounted noexec](https://github.com/rust-lang-nursery/rustup.rs/pull/523).
* [Remove printfs from ~/.cargo/env](https://github.com/rust-lang-nursery/rustup.rs/pull/527).
* [Reduce margin in installer text to 79 columns](https://github.com/rust-lang-nursery/rustup.rs/pull/526).
* [Fix typos](https://github.com/rust-lang-nursery/rustup.rs/pull/519).
* [Fix missing curly braces in error-chain docs](https://github.com/rust-lang-nursery/rustup.rs/pull/522).
* [Fix downloads of builds without v2 manifests](https://github.com/rust-lang-nursery/rustup.rs/pull/515).
* [Explain toolchains in `help install`](https://github.com/rust-lang-nursery/rustup.rs/pull/496).
* [Compile on stable Rust](https://github.com/rust-lang-nursery/rustup.rs/pull/476).
* [Fix spelling mistakes](https://github.com/rust-lang-nursery/rustup.rs/pull/489).
* [Fix the toolchain command synonyms](https://github.com/rust-lang-nursery/rustup.rs/pull/477).
* [Configurable host triples](https://github.com/rust-lang-nursery/rustup.rs/pull/421).
* [Use a .toml file to store settings](https://github.com/rust-lang-nursery/rustup.rs/pull/420).
* [Point PATH to toolchain/bin on Windows](https://github.com/rust-lang-nursery/rustup.rs/pull/402).
* [Remove extra '.' in docs](https://github.com/rust-lang-nursery/rustup.rs/pull/472).

Contributors: Alex Crichton, benaryorg, Benedikt Reinartz, Boutin,
Michael, Brian Anderson, Diggory Blake, Erick Tryzelaar, Ivan
Nejgebauer, Jeremiah Peschka, Josh Stone, Knight, mdinger, Ryan Kung,
Tad Hardesty

# 0.1.12

* [Don't install when multirust metadata exists](https://github.com/rust-lang-nursery/rustup.rs/pull/456).

# 0.1.11

* [Actually dispatch the `rustup install` command](https://github.com/rust-lang-nursery/rustup.rs/pull/444).
* [Migrate to libcurl instead of hyper](https://github.com/rust-lang-nursery/rustup.rs/pull/434).
* [Add error for downloading bogus versions](https://github.com/rust-lang-nursery/rustup.rs/pull/428).

# 0.1.10

* [Multiple cli improvements](https://github.com/rust-lang-nursery/rustup.rs/pull/419).
* [Support HTTP protocol again](https://github.com/rust-lang-nursery/rustup.rs/pull/431).
* [Improvements to welcome screen](https://github.com/rust-lang-nursery/rustup.rs/pull/418).
* [Don't try to update non-tracking channels](https://github.com/rust-lang-nursery/rustup.rs/pull/425).
* [Don't panic when NativeSslStream lock is poisoned](https://github.com/rust-lang-nursery/rustup.rs/pull/429).
* [Fix multiple issues in schannel bindings](https://github.com/sfackler/schannel-rs/pull/1)

# 0.1.9

* [Do TLS hostname verification](https://github.com/rust-lang-nursery/rustup.rs/pull/400).
* [Expand `rustup show`](https://github.com/rust-lang-nursery/rustup.rs/pull/406).
* [Add `rustup doc`](https://github.com/rust-lang-nursery/rustup.rs/pull/403).
* [Refuse to install if it looks like other Rust installations are present](https://github.com/rust-lang-nursery/rustup.rs/pull/408).
* [Update www platform detection for FreeBSD](https://github.com/rust-lang-nursery/rustup.rs/pull/399).
* [Fix color display during telemetry capture](https://github.com/rust-lang-nursery/rustup.rs/pull/394).
* [Make it less of an error for the self-update hash to be wrong](https://github.com/rust-lang-nursery/rustup.rs/pull/372).

# 0.1.8

* [Initial telemetry implementation (disabled)](https://github.com/rust-lang-nursery/rustup.rs/pull/289)
* [Add hash to `--version`](https://github.com/rust-lang-nursery/rustup.rs/pull/347)
* [Improve download progress](https://github.com/rust-lang-nursery/rustup.rs/pull/355)
* [Completely overhaul error handling](https://github.com/rust-lang-nursery/rustup.rs/pull/358)
* [Add armv7l support to www](https://github.com/rust-lang-nursery/rustup.rs/pull/359)
* [Overhaul website](https://github.com/rust-lang-nursery/rustup.rs/pull/363)

# 0.1.7

* [Fix overrides for Windows root directories](https://github.com/rust-lang-nursery/rustup.rs/pull/317).
* [Remove 'multirust' binary and rename crates](https://github.com/rust-lang-nursery/rustup.rs/pull/312).
* [Pass rustup-setup.sh arguments to rustup-setup](https://github.com/rust-lang-nursery/rustup.rs/pull/325).
* [Don't open /dev/tty if passed -y](https://github.com/rust-lang-nursery/rustup.rs/pull/334).
* [Add interactive install, `--default-toolchain` argument](https://github.com/rust-lang-nursery/rustup.rs/pull/293).
* [Rename rustup-setup to rustu-init](https://github.com/rust-lang-nursery/rustup.rs/pull/303).
