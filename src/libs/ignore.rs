use globset::GlobBuilder;
use ignore::WalkBuilder;
use std::io::Error;
use std::path::PathBuf;

pub fn match_glob_and_ignore(glob: String) -> Result<Vec<PathBuf>, Error> {
    let walker = WalkBuilder::new("./").threads(0).build();

    let mut entries: Vec<PathBuf> = vec![];

    let glob_builder = match GlobBuilder::new(&glob).literal_separator(true).build() {
        Ok(glob_builder) => glob_builder,
        Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e)),
    };

    let glob_matcher = glob_builder.compile_matcher();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path().to_str().unwrap_or("");
                if glob_matcher.is_match(path) && entry.path().is_file() {
                    entries.push(entry.path().to_path_buf());
                }
            }
            Err(e) => {
                return Err(Error::new(std::io::ErrorKind::Other, e));
            }
        }
    }
    Ok(entries)
}
