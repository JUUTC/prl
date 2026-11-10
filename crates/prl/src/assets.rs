//! Explicit development asset opt-in; packaged builds never search the checkout.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

pub(super) fn content_path() -> io::Result<PathBuf> {
    let development = development_assets(std::env::args_os().skip(1))?;
    let executable = std::env::current_exe()?;
    resolve_content_path(&executable, development)
}

fn development_assets(arguments: impl IntoIterator<Item = OsString>) -> io::Result<bool> {
    let arguments: Vec<_> = arguments.into_iter().collect();
    match arguments.as_slice() {
        [] => Ok(false),
        [argument] if argument == "--dev-assets" => Ok(true),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: prl [--dev-assets]; --dev-assets opts into source-checkout assets",
        )),
    }
}

fn resolve_content_path(executable: &Path, development: bool) -> io::Result<PathBuf> {
    let assets = executable
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent directory"))?
        .join("assets");
    // An incomplete adjacent package must fail, even with development opted in.
    if !development || assets.try_exists()? {
        return Ok(assets.join("packs/prl/base.ron"));
    }
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/prl/base.ron"))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    struct PackageDirectory(PathBuf);

    impl PackageDirectory {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "prl-assets-test-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn executable(&self) -> PathBuf {
            self.0.join("prl.exe")
        }
    }

    impl Drop for PackageDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn checkout_assets_require_explicit_opt_in() {
        assert!(!development_assets([]).unwrap());
        assert!(development_assets([OsString::from("--dev-assets")]).unwrap());
        assert!(development_assets([OsString::from("--dev-asset")]).is_err());
        assert!(
            development_assets([
                OsString::from("--dev-assets"),
                OsString::from("--dev-assets"),
            ])
            .is_err()
        );
    }

    #[test]
    fn missing_packaged_assets_never_fall_back_to_existing_checkout() {
        let package = PackageDirectory::new();
        let path = resolve_content_path(&package.executable(), false).unwrap();
        assert_eq!(path, package.0.join("assets/packs/prl/base.ron"));
        assert!(crate::load_playtest(&path).is_err());
    }

    #[test]
    fn development_opt_in_loads_checkout_when_no_adjacent_assets_exist() {
        let package = PackageDirectory::new();
        let path = resolve_content_path(&package.executable(), true).unwrap();
        assert!(crate::load_playtest(&path).is_ok());
    }

    #[test]
    fn incomplete_adjacent_assets_never_fall_back_even_with_development_opt_in() {
        let package = PackageDirectory::new();
        std::fs::create_dir(package.0.join("assets")).unwrap();
        for development in [false, true] {
            let path = resolve_content_path(&package.executable(), development).unwrap();
            assert_eq!(path, package.0.join("assets/packs/prl/base.ron"));
            assert!(crate::load_playtest(&path).is_err());
        }
    }

    #[test]
    fn adjacent_content_is_loaded_without_source_checkout_fallback() {
        let package = PackageDirectory::new();
        let pack = package.0.join("assets/packs/prl");
        std::fs::create_dir_all(&pack).unwrap();
        let source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/prl/base.ron");
        std::fs::copy(source, pack.join("base.ron")).unwrap();
        let dungeon = pack.join("data/dungeons/crypt_playtest.ron");
        std::fs::create_dir_all(dungeon.parent().unwrap()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/packs/prl/data/dungeons/crypt_playtest.ron");
        std::fs::copy(source, &dungeon).unwrap();
        for development in [false, true] {
            let path = resolve_content_path(&package.executable(), development).unwrap();
            assert_eq!(path, pack.join("base.ron"));
            assert!(crate::load_playtest(&path).is_ok());
        }
        std::fs::remove_file(&dungeon).unwrap();
        let error = crate::load_playtest(&pack.join("base.ron"))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("crypt_playtest.ron"));
        std::fs::write(&dungeon, "invalid required dungeon").unwrap();
        let error = crate::load_playtest(&pack.join("base.ron"))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("crypt_playtest.ron"));
        std::fs::write(pack.join("base.ron"), "invalid content").unwrap();
        for development in [false, true] {
            let path = resolve_content_path(&package.executable(), development).unwrap();
            assert!(crate::load_playtest(&path).is_err());
        }
    }
}
