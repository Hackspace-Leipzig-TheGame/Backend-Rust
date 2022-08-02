{
  inputs = {
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs = inputs:
    inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      defaultOutputs = {
        app = "the-game-backend";
        package = "the-game-backend";
      };
    };
}
