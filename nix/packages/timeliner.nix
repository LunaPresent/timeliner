{
  runCommand,
  rustPlatform,
  writers,
  lib,
}:
let
  src = ../../.;
  manifest = lib.importTOML "${src}/Cargo.toml";

  package = rustPlatform.buildRustPackage {
    inherit src;
    inherit (manifest.package) version;

    pname = manifest.package.name;
    cargoLock.lockFile = "${src}/Cargo.lock";
    passthru = { inherit generate; };

    meta = {
      description = "Create a topological timeline from a list of dated and ordered events";
      platforms = lib.platforms.all;
      maintainers = [ lib.maintainers.toodeluna ];
    };
  };

  generate =
    input:
    let
      configFile = writers.writeTOML "input.toml" input;
    in
    runCommand "timeliner-generate" { nativeBuildInputs = [ package ]; } ''
      mkdir -p "$out"
      cat "${configFile}" | timeliner -o html-css > "$out/timeline.html"
    '';
in
package
