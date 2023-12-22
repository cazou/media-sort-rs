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

    /// Do not actually move files, only list the actions that would be taken
    #[structopt(long)]
    dry_run: bool,

    /// Sort the give folder instead of watching the configured folder.
    /// The program will close once the media files have been sorted.
    #[structopt(long)]
    sort: Option<PathBuf>,

    /// Check the given folder for files that would be sorted in the same files.
    /// This is useful to be sure that a better version of a show will not be overwritten by a less
    /// good one.
    /// This will show a summary of which files are conflicting and which file could not be sorted
    /// because it was not found online
    /// Nothing will be moved (--dry-run has no effect)
    #[structopt(long)]
    check: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let config = match config::Config::from_file(&opts.config) {
        Ok(c) => c,
        Err(e) => bail!("Cannot open {:?}: {e}", &opts.config),
    };

    let mut sorter = mediasort::MediaSort::new(config, opts.dry_run)?;

    if opts.sort.is_some() {
        sorter.sort(&opts.sort.unwrap(), opts.dry_run)
    } else if opts.check.is_some() {
        sorter.check(&opts.check.unwrap())
    } else {
        sorter.watch()
    }
}
