{ self, lib, ... }:
{
  perSystem =
    { self', pkgs, ... }:
    {
      packages = {
        default = pkgs.callPackage "${self}/nix/packages/timeliner.nix" { };
        example = self'.packages.default.generate (lib.importTOML "${self}/example/example-input.toml");
      };
    };
}
