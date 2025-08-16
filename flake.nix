{
  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable"; };

  outputs = { nixpkgs, ... }:
    let
      supportSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportSystems;
    in
    {
      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixpkgs-fmt);

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              pkg-config
              llvm_18.dev
              libffi
              libxml2
            ];
            shellHook = ''
              export LLVM_SYS_181_PREFIX=${pkgs.llvm_18.dev}
            '';
          };
        }
      );
    };
}

