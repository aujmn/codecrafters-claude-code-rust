use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Args {
    file_path: String,
}

pub(crate) fn read(args: Args) -> std::io::Result<String> {
    std::fs::read_to_string(args.file_path)
}
