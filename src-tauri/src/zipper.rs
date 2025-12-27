use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

pub struct Zipper {
    pub path: String,
}

impl Zipper {
    pub fn new(path: String) -> Self {
        Zipper { path: path }
    }

    fn sanitize_path(&self, name: &str, base: &Path) -> io::Result<PathBuf> {
        let mut out = base.to_path_buf();

        for component in Path::new(name).components() {
            match component {
                std::path::Component::Normal(c) => out.push(c),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid ZIP path",
                    ))
                }
            }
        }

        Ok(out)
    }

    pub fn unzip_temporary(&self) -> Result<PathBuf, String> {
        let zip_path = PathBuf::from(&self.path);

        if zip_path.extension().and_then(|e| e.to_str()) != Some("zip") {
            return Err(format!("File {:?} is not a zip archive", zip_path));
        }

        let exe_dir = env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Unable to get executable directory")?
            .to_path_buf();

        let target_dir = exe_dir.join("esptool-gui-temp");

        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Unable to create target dir {:?}: {}", target_dir, e))?;
    {
        let file = File::open(&zip_path)
            .map_err(|e| format!("Unable to open zip {:?}: {}", zip_path, e))?;

        let file_ref = &file;

        let mut archive = ZipArchive::new(file_ref)
            .map_err(|e| format!("Invalid zip archive {:?}: {}", zip_path, e))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;

            let relative = Path::new(entry.name())
                .components()
                .skip(1)
                .collect::<PathBuf>();

            if relative.as_os_str().is_empty() {
                continue;
            }

            let out_path = self
                .sanitize_path(relative.to_str().unwrap(), &target_dir)
                .map_err(|e| e.to_string())?;

            if entry.is_dir() {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;

                io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
            }
        }
    }
        Ok(target_dir)
    }

    pub fn remove_temporary(&self, path: String) -> Result<(), io::Error> {
        let path = Path::new(&path);
        if !path.exists() {
            return Ok(());
        }

        for _ in 0..5 {
            match fs::remove_dir_all(path) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) => {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        return fs::remove_dir_all(path);
    }
}
