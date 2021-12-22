{
  fetchurl = { name ? "source", sha256, url, system }:
    builtins.derivation {
      inherit name system;
      nixek_fetcher_url = url;
      builder = ./nixek-fetcher;
      outputHashMode = "flat";
      outputHashAlgo = "sha256";
      outputHash = sha256;
    };

  fetchTarball = { name ? "source", sha256, url, system }:
    builtins.derivation {
      inherit name system;
      nixek_fetcher_url = url;
      nixek_fetcher_unpack = "1";
      builder = ./nixek-fetcher;
      outputHashMode = "recursive";
      outputHashAlgo = "sha256";
      outputHash = sha256;
    };
}
