use crate::config;
use crate::mediainfo::MediaInfo;
use anyhow::bail;
use libc;
use libc::c_char;
use notify::event::AccessKind;
use notify::event::ModifyKind::Name;
use notify::{
    Config, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

pub(crate) struct MediaSort {
    rx: Receiver<Result<Event>>,
    config: config::Config,
    dry_run: bool,
    checked: HashMap<PathBuf, Vec<PathBuf>>,
    _watcher: INotifyWatcher,
}

impl MediaSort {
    pub fn new(config: config::Config, dry_run: bool) -> anyhow::Result<MediaSort> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(&config.dir_watch), RecursiveMode::Recursive)?;

        Ok(MediaSort {
            rx,
            config,
            dry_run,
            checked: HashMap::new(),
            _watcher: watcher,
        })
    }

    pub fn watch(&mut self) -> anyhow::Result<()> {
        for res in &self.rx {
            match res {
                Ok(e) => Self::process_event(&e, &self.config, self.dry_run),
                Err(e) => anyhow::bail!("watch error: {:?}", e),
            }
        }

        Ok(())
    }

    pub fn sort(&self, path: &PathBuf, dry_run: bool) -> anyhow::Result<()> {
        for entry in path.read_dir()? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.sort(&entry.path(), dry_run)?;
            } else {
                match Self::process_file(&entry.path(), &self.config, dry_run) {
                    Ok(p) => println!("Sorted {entry:?} to {p:?}"),
                    Err(e) => println!("Cannot sort {entry:?}: {e}"),
                }
            }
        }

        Ok(())
    }

    fn do_check(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        for entry in path.read_dir()? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.do_check(&entry.path())?;
            } else {
                match Self::process_file(&entry.path(), &self.config, true) {
                    Ok(p) => {
                        let paths = self.checked.entry(p).or_insert(vec![]);
                        paths.push(path.clone());
                    }
                    Err(e) => println!("Cannot check {entry:?}: {e}"),
                }
            }
        }

        Ok(())
    }

    pub fn check(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let res = self.do_check(path);
        for (k, v) in &self.checked {
            if v.len() > 1 {
                println!("{k:?} -> {v:?}");
            }
        }

        res
    }

    /// Process the given new_file
    /// If dry_run is true, the action will be logged but not executed.
    /// Returns an error or the destination path once processed
    fn process_file(
        new_file: &PathBuf,
        config: &config::Config,
        dry_run: bool,
    ) -> anyhow::Result<PathBuf> {
        let info = MediaInfo::from_path(&new_file, &config.omdb.apikey)?;

        let mut dst = if info.is_show() {
            let show = info.show_info.unwrap();
            let mut show_path = config.show_path.join(info.name.clone());

            // Check if the path exists without a year
            show_path = if show_path.exists() || info.year.is_none() {
                show_path
            } else {
                config
                    .show_path
                    .join(format!("{} ({})", info.name.clone(), info.year.unwrap()))
            };

            show_path
                .join(format!("Season {:02}", show.season))
                .join(format!(
                    "{} - S{:02}E{}",
                    info.name.clone(),
                    show.season,
                    show.episode
                ))
        } else {
            config.movie_path.join(match info.year {
                Some(y) => format!("{} ({})", info.name.clone(), y),
                None => info.name.clone(),
            })
        };

        if let Some(e) = new_file.extension() {
            dst.set_extension(e);
        }

        if dst.exists() && !config.overwrite {
            bail!("{dst:?} already exists: Skipping")
        }

        println!("move {:?} to {:?}", new_file, dst);

        if dry_run {
            return Ok(dst);
        }

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::rename(&new_file, &dst) {
            Err(_) => {
                // Maybe new_file and dst are on different FS, try to copy the file instead.
                fs::copy(&new_file, &dst)?;
                fs::remove_file(&new_file)?;
            }
            _ => {}
        }

        let ret = dst.clone();

        // Set the permissions
        while dst != config.show_path && dst != config.movie_path {
            let mut perms = fs::metadata(&dst)?.permissions();
            let mode = config.permissions.mode + if dst.is_dir() { 0o111 } else { 0 };
            perms.set_mode(mode);
            fs::set_permissions(&dst, perms)?;

            // Set user/group
            unsafe {
                let pwd = libc::getpwnam(config.permissions.user.as_ptr() as *const c_char);

                if !pwd.is_null() {
                    let pwd = *pwd;

                    libc::chown(
                        dst.to_str().unwrap().as_ptr() as *const c_char,
                        pwd.pw_uid,
                        pwd.pw_gid,
                    );
                }
            }

            dst.pop();
        }

        Ok(ret)
    }

    fn process_event(e: &Event, config: &config::Config, dry_run: bool) {
        // FIXME: Sometimes, it looks like folders are just created...
        match e.kind {
            EventKind::Access(AccessKind::Close(_)) => {
                println!("closed: {:?}", e.paths[0]);
                if let Err(err) = Self::process_file(&e.paths[0], config, dry_run) {
                    println!("Cannot process {:?}: {err}. Ignoring...", e.paths[0]);
                    // TODO: There should be a way to notify the issue
                    //       A nice way would be via Home assistant
                }
            }
            EventKind::Modify(Name(notify::event::RenameMode::To)) => {
                println!("Renamed: {:?}", e.paths[0]);
                if let Err(err) = Self::process_file(&e.paths[0], config, dry_run) {
                    println!("Cannot process {:?}: {err}. Ignoring...", e.paths[0]);
                    // TODO: There should be a way to notify the issue
                    //       A nice way would be via Home assistant
                }
            }
            _ => {}
        }
    }
}
