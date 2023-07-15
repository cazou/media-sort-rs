use crate::config;

use crate::mediainfo::MediaInfo;
use crate::mediainfo::MediaInfo::{Movie, NoMedia, TVShow};
use notify::event::AccessKind;
use notify::{
    Config, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

pub(crate) struct MediaSort {
    created_files: Vec<PathBuf>,
    rx: Receiver<Result<Event>>,
    config: config::Config,
    _watcher: INotifyWatcher,
}

impl MediaSort {
    pub fn new(config: config::Config) -> anyhow::Result<MediaSort> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(&config.dir_watch), RecursiveMode::Recursive)?;

        Ok(MediaSort {
            created_files: vec![],
            rx,
            config,
            _watcher: watcher,
        })
    }

    pub fn watch(&mut self) -> anyhow::Result<()> {
        for res in &self.rx {
            match res {
                Ok(e) => Self::process_event(&e, &mut self.created_files, &self.config),
                Err(e) => anyhow::bail!("watch error: {:?}", e),
            }
        }

        Ok(())
    }

    fn process_file(new_file: &PathBuf, config: &config::Config) -> anyhow::Result<()> {
        let mut dst = match MediaInfo::from_path(&new_file, &config.omdb.apikey)? {
            TVShow { info } => config
                .show_path
                .join(info.title.clone())
                .join(format!("Season {:02}", info.season))
                .join(format!(
                    "{} - S{:02}E{:02}",
                    info.title.clone(),
                    info.season,
                    info.episode
                )),
            Movie { info } => {
                config
                    .movie_path
                    .join(format!("{} ({})", info.title.clone(), info.year.clone()))
            }
            NoMedia { path } => {
                println!("{:?}: Not a media file. Ignored", path);
                return Ok(());
            }
        };

        if let Some(e) = new_file.extension() {
            dst.set_extension(e);
        }

        println!("move {:?} to {:?}", new_file, dst);

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&new_file, &dst)?;
        fs::remove_file(&new_file)?;

        // Set the permissions
        while dst != config.show_path && dst != config.movie_path {
            let mut perms = fs::metadata(&dst)?.permissions();
            let mode = config.permissions.mode + if dst.is_dir() { 0o111 } else { 0 };
            perms.set_mode(mode);
            fs::set_permissions(&dst, perms)?;

            dst.pop();
        }

        Ok(())
    }

    fn process_event(e: &Event, created_files: &mut Vec<PathBuf>, config: &config::Config) {
        match e.kind {
            EventKind::Create(_) => {
                println!("created: {:?}", e.paths[0]);
                created_files.push(e.paths[0].clone());
            }
            EventKind::Access(AccessKind::Close(_)) => {
                println!("closed: {:?}", e.paths[0]);
                if created_files.contains(&e.paths[0]) {
                    if let Err(err) = Self::process_file(&e.paths[0], config) {
                        println!("Cannot process {:?}: {err}. Ignoring...", e.paths[0]);
                        // TODO: There should be a way to notify the issue
                        //       A nice way would be via Home assistant
                    }
                }
            }
            _ => {}
        }
    }
}
