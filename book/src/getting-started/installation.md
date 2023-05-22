## 🌞 Installation

Packages marked with 🔸 are maintained by community members (i.e. not the cargo-msrv authors). A big thank you to them!

### Using [Cargo](https://doc.rust-lang.org/cargo/commands/cargo-install.html):

You can install cargo-msrv from source by using Cargo, the Rust package manager and build tool ([package](https://crates.io/crates/cargo-msrv)).

**How to install the latest stable release?**

```shell
cargo install cargo-msrv
```

**How to install the latest stable release more quickly?**

Similar to the above, but allows for only the default channel to obtain a list of rustc releases.
This compiles about 40% faster and produces binaries about half the size in the range of 4.5MB.

```shell
cargo install cargo-msrv --no-default-features
```

**How to install the latest development release?**

You may install _cargo-msrv_ from GitHub:

```shell
cargo install cargo-msrv --git https://github.com/foresterre/cargo-msrv.git --branch main
```

### Arch Linux 🔸

cargo-msrv is available from the Arch Linux [extra repository](https://archlinux.org/packages/extra/x86_64/cargo-msrv/).

**How to install?**

```shell
pacman -S cargo-msrv
```

### Nix 🔸

cargo-msrv is available from the Nix package manager and in NixOS ([package](https://search.nixos.org/packages?channel=21.05&show=cargo-msrv&from=0&size=50&sort=relevance&type=packages&query=cargo-msrv)):

**How to install (nixpkgs)?**

```shell
nix-env -iA nixpkgs.cargo-msrv
```

**How to install (NixOS)?**

```shell
nix-env -iA nixos.cargo-msrv
```

NB: When installing with `nix-shell --pure`, ensure that `rustup` is available in the environment.

### Other options

You may also build the program from source by cloning the [repository](https://github.com/foresterre/cargo-msrv)
and building a release from there.

**How to build a release?**

```shell
git clone git@github.com:foresterre/cargo-msrv.git
git checkout v0.16.0 # NB: Find the latest release tag here: https://github.com/foresterre/cargo-msrv/tags
cd cargo-msrv
cargo install cargo-msrv --path . # OR cargo build --release && mv ./target/cargo-msrv ./my/install/directory
```

**How to build the latest development version from source?**

```shell
git clone git@github.com:foresterre/cargo-msrv.git
cd cargo-msrv
cargo install cargo-msrv --path . # OR cargo build --release && mv ./target/cargo-msrv ./my/install/directory
```


You may find additional installation options in the [README](https://github.com/foresterre/cargo-msrv#install).
