let
	pkgs = import (fetchTarball "https://github.com/nixos/nixpkgs/archive/master.tar.gz") {};
	fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };

	rustToolchain = 
		fenix.complete.withComponents [
			"cargo"
			"clippy"
			"miri"
			"rustc"
			"rust-analyzer"
			"rustfmt"
		];
in
pkgs.mkShell {
	nativeBuildInputs = [
		rustToolchain
		pkgs.qemu
	];
}
