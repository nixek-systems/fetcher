use tokio::fs::File;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Url to fetch
    let url = std::env::var("nixek_fetcher_url").expect("env var 'nixek_fetcher_url' must be set.");
    // Whether to unpack into out vs just fetching the url, i.e. 'fetchTarball' vs 'fetchurl'
    let unpack = std::env::var("nixek_fetcher_unpack").map(parse_env_str).unwrap_or(false);
    // out path
    let out = std::env::var("out").expect("env var 'out' should be set for a nix builder");

    // And now fetch
    let res = reqwest::get(url).await?.error_for_status()?;
    let body = res.bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read();

    if !unpack {
        let mut f = File::create(out).await.expect("could not create out file");
        tokio::io::copy(&mut body.compat(), &mut f).await.expect("unable to write full body to out file");
        return Ok(())
    }

    // unpack == true

    panic!("TODO: unpack");

    Ok(())
}

fn parse_env_str(s: String) -> bool {
    match s.as_str() {
        "false" | "FALSE" | "False" | "0" | "no" | "n" => false,
        _ => true,
    }
}
