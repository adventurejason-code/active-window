{
  description = "Prints the currently active Wayland window title to stdout";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      mkPackage = system:
        nixpkgs.legacyPackages.${system}.rustPlatform.buildRustPackage {
          pname = "active-window";
          version = "0.1.1";
          src = self;
          cargoHash = "sha256-fqPByh7xVTCzIiPc6pEvuCkOHhU9+k69cWBc2URVOw4=";
          meta = with nixpkgs.legacyPackages.${system}.lib; {
            description = "Prints the currently active Wayland window title to stdout";
            homepage = "https://github.com/adventurejason-code/active-window";
            license = licenses.mit;
            mainProgram = "active-window";
          };
        };
    in {
      packages.x86_64-linux.default = mkPackage "x86_64-linux";
      packages.aarch64-linux.default = mkPackage "aarch64-linux";
    };
}
