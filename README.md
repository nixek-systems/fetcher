### fetcher

nixek-fetcher is a statically compiled rust binary capable of downloading urls
and tarballs. It comes with a `default.nix` that provides the familiar
'fetchurl' and 'fetchTarball' functions that you may be used to from the nix
builtins.

#### Why might you want to use this?

In general, you should use the more standardized [fetchers](https://nixos.org/manual/nixpkgs/stable/#chap-pkgs-fetchers) in nixpkgs.

However, there are cases where you may not yet have a local nixpkgs reference handy, and need to fetch it... which leaves you with a chicken and egg problem.

Often, this problem is resolved with `builtins.fetchTarball`, but `builtins.fetchTarball` can be less than ideal, for example due to [not actually performing substitutes](https://github.com/NixOS/nix/issues/2114).

The above is actually the motivating use-case: I have an extracted `nixpkgs-source` in my nix store, but `builtins.fetchTarball` ignores the provided sha256 and still redownloads it.

This fetcher is intended to be a reasonable bootstrapping step between builtins.fetchTarball and downloading all of nixpkgs.

To better explain the motivating issue above, consider the following examples on my machine:

```
$ cat ./example.nix
let
  system = "x86_64-linux";
  nixek-fetchers = import (builtins.fetchTarball {
    url = <url to release tarball omitted>;
    sha256 = "1rwwgcbj9v0rp3d2nappq7qqh49cg3vdi1zqxvi5y24bmg07yqf7";
  });
in
{
  nixek = (import (nixek-fetchers.fetchTarball {
    inherit system;
    url = "https://github.com/NixOS/nixpkgs/archive/c6dcfb8f7b4532cdaa42164cd48129139378b9e7.tar.gz";
    sha256 = "sha256-8Sk9dw4EbTBocFf+72lNgiWqECXSnzNfHYIVj/qelR8=";
  }) {inherit system;}).ddate;

  nix-builtin = (import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/c6dcfb8f7b4532cdaa42164cd48129139378b9e7.tar.gz";
    sha256 = "sha256-8Sk9dw4EbTBocFf+72lNgiWqECXSnzNfHYIVj/qelR8=";
  }) {inherit system;}).ddate;
}

# Ensure nixpkgs store-path is present
$ nix-prefetch-url https://github.com/NixOS/nixpkgs/archive/c6dcfb8f7b4532cdaa42164cd48129139378b9e7.tar.gz --unpack --name source
path is '/nix/store/myn6gq70iwqx3h8ajcw5l010hsd38ar2-source'
07wmkvx8y5c23mgk77yj4l8al9c29mlyzzjpf1l30v841rvksagi
# And prefetch ddate from the binary cache so we're not measuring that
$ nix-build -A ddate /nix/store/myn6gq70iwqx3h8ajcw5l010hsd38ar2-source

# Pretend we did a nix-copy-closure or such to get this store path on a new
# machine where the tarball cache doesn't exist
$ rm -f ~/.cache/nix/tarballs

# nixek-fetchers.fetchTarball

$ systemd-run --pipe --wait -p IPAccounting=true -- nix-build /path/to/example.nix -A nixek
/nix/store/cql8qllb1fc3gvkl6dx3k9aacxaxqcvq-ddate-0.2.2
Finished with result: success
Main processes terminated with: code=exited/status=0
Service runtime: 528ms
CPU time consumed: 272ms
IP traffic received: 4.4M
IP traffic sent: 206B

---

# builtins.fetchTarball

$ systemd-run --pipe --wait -p IPAccounting=true -- nix-build /path/to/example.nix -A nix-builtin
/nix/store/cql8qllb1fc3gvkl6dx3k9aacxaxqcvq-ddate-0.2.2
Finished with result: success
Main processes terminated with: code=exited/status=0
Service runtime: 8.727s
CPU time consumed: 4.343s
IP traffic received: 28.2M
IP traffic sent: 629.6K
```

As we can see in this example, using nixek-fetchers with a prepopulated local
nix store resulted in a 500ms runtime to "do nothing" (verify we already have
the package in the store). The majority of the time was spent downloading the
4.4MB "nixek-fetcher" rust binary, which was then not run.

On the other hand, using builtins.fetchTarball to do nothing took over 10 times
longer (at 8.7s), including downloading and extracting considerably more data.
