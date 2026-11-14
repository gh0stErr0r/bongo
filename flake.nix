{
  description = "Build a bongo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = {
    crane,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-darwin"];
      perSystem = {pkgs, ...}: let
        craneLib = crane.mkLib pkgs;
        src = ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            if pkgs.stdenv.hostPlatform.isDarwin
            then [pkgs.gtk4]
            else [];
          nativeBuildInputs =
            if pkgs.stdenv.hostPlatform.isDarwin
            then [pkgs.pkg-config]
            else [];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        crate = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in {
        formatter = pkgs.alejandra;
        packages.default = crate;
        devShells.default = craneLib.devShell {};
      };
    };
}
