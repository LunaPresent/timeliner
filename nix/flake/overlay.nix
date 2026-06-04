{ withSystem, ... }:
{
  flake.overlays.default = final: prev: {
    timeliner = withSystem prev.stdenv.hostPlatform.system ({ self', ... }: self'.packages.default);
  };
}
