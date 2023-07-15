mod config;
mod mediainfo;
mod mediasort;
mod omdb;
mod tvmaze;

use anyhow::bail;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, default_value = "/etc/media-sort-rs.yaml")]
    config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let config = match config::Config::from_file(&opts.config) {
        Ok(c) => c,
        Err(e) => bail!("Cannot open {:?}: {e}", &opts.config),
    };

    let mut sorter = mediasort::MediaSort::new(config)?;
    sorter.watch()
}
