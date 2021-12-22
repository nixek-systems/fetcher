use std::fs::File;
use std::path::{Path, PathBuf, Component};

use reqwest::blocking::get;
use tempdir::TempDir;
use flate2::read::GzDecoder;

fn main() -> Result<(), reqwest::Error> {
    // Url to fetch
    let url = std::env::var("nixek_fetcher_url").expect("env var 'nixek_fetcher_url' must be set.");
    // Whether to unpack into out vs just fetching the url, i.e. 'fetchTarball' vs 'fetchurl'
    let unpack = std::env::var("nixek_fetcher_unpack").map(parse_env_str).unwrap_or(false);
    // out path
    let out = std::env::var("out").expect("env var 'out' should be set for a nix builder");

    // And now fetch
    let mut res = get(url)?.error_for_status()?;
    let content_type = res.headers().get(reqwest::header::CONTENT_TYPE).map(|v| v.to_owned()).map(|v| v.to_str().expect("invalid content-type header").to_string()).unwrap_or("".to_string());

    if !unpack {
        let mut f = File::create(&out).expect("could not create out file");
        res.copy_to(&mut f).expect("unable to write full body to out file");
        return Ok(())
    }

    // unpack == true
    // For now, only support tarballs that the tar crate does; add zip etc later

    let tmp = TempDir::new("nixek-fetcher").expect("error making tempdir");
    let archive_path = tmp.path().join("archive.tar");

    match content_type.as_str() {
        "application/x-gzip" | "application/gzip" | "application/x-gtar" | "application/x-tgz" => {
            let mut gunzipper = GzDecoder::new(res);
            let mut f = File::create(&archive_path).expect("error creating temporary file to download archive");
            std::io::copy(&mut gunzipper, &mut f).expect("unable to write gzipped archive");
        },
        "application/tar" | "application/x-tar" | "" => {
            let mut f = File::create(&archive_path).expect("error creating temporary file to download archive");
            res.copy_to(&mut f).expect("unable to write full body to temporary file");
        },
        _ => {
            panic!("unrecognized content type for unpack fetcher: {}", content_type);
        }
    }

    let stdfi = std::fs::File::open(&archive_path).expect("could not open file we just wrote");

    let mut archive = tar::Archive::new(stdfi);

    // Nix builtins.fetchTarball does effectively --strip-components=1 https://github.com/NixOS/nix/blob/dc83298449d5547926febe24c7bf973341127f60/src/libfetchers/tarball.cc#L144-L148
    // Mimic that ourselves too.
    unpack_strip_components(&mut archive, 1, &Path::new(&out)).expect("could not unpack archive");
    Ok(())
}

fn parse_env_str(s: String) -> bool {
    match s.as_str() {
        "false" | "FALSE" | "False" | "0" | "no" | "n" => false,
        _ => true,
    }
}

// This code is derived from tar-rs code, used under the terms of the MIT license
// (Copyright (c) 2014 Alex Crichton).
// https://github.com/alexcrichton/tar-rs
fn unpack_strip_components<R: std::io::Read>(a: &mut tar::Archive<R>, components: usize, dst: &Path) -> Result<(), std::io::Error> {
    let entries = a.entries().expect("could not get entries");
    std::fs::create_dir_all(&dst).expect("could not create destination");

    let dst = &dst.canonicalize().unwrap_or(dst.to_path_buf());

    let mut directories = Vec::new();
    'next_entry: for entry in entries {
        let mut file = entry.expect("error getting entry");
        let path = file.path().expect("could not get entry path");

        let mut normalized_path = std::path::PathBuf::new();
        for c in path.components() {
            match c {
                Component::Prefix(..) | Component::RootDir | Component::CurDir => continue,
                Component::ParentDir => {
                    continue 'next_entry;
                },
                Component::Normal(part) => normalized_path.push(part),
            }
        }

        let short_name: PathBuf = normalized_path.components().skip(components).collect();

        if short_name.components().count() == 0 {
            continue 'next_entry;
        }

        let file_dst = dst.join(short_name);

        std::fs::create_dir_all(file_dst.parent().expect("must have parent to unpack")).expect("could not create directory for tar extraction");
        if file.header().entry_type() == tar::EntryType::Directory {
            directories.push((file, file_dst));
        } else {
            file.unpack(file_dst)?;
        }
    }

    for mut dir in directories {
        dir.0.unpack(dir.1).expect("could not write directory metadata");
    }
    Ok(())
}
