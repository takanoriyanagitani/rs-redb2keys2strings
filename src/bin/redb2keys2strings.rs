use std::io;
use std::process::ExitCode;

use rs_redb2keys2strings::tabname2path2db2keys2vals_stdio;

fn io_envkey2val(key: &'static str) -> impl Fn() -> String {
    move || std::env::var(key).unwrap_or_default()
}

fn io_table_name() -> impl Fn() -> String {
    io_envkey2val("ENV_TABLE_NAME")
}

fn io_db_path() -> impl Fn() -> String {
    io_envkey2val("ENV_REDB_PATH")
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let tabname: String = io_table_name()();
        let db_path: String = io_db_path()();
        let mut path2keys2vals2json = tabname2path2db2keys2vals_stdio(tabname);
        path2keys2vals2json(db_path)
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let tabname: String = io_table_name()();
        let db_path: String = io_db_path()();

        eprintln!("ENV_TABLE_NAME: {tabname}");
        eprintln!("ENV_REDB_PATH: {db_path}");

        ExitCode::FAILURE
    })
}
