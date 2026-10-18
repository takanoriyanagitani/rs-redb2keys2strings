use std::io;
use std::path::Path;

use io::BufWriter;
use io::Write;

use io::BufRead;

use redb::ReadOnlyTable;
use redb::ReadTransaction;

use redb::ReadOnlyDatabase;
use redb::ReadableDatabase;

use redb::TableDefinition;

pub struct Wtr<W> {
    pub wtr: W,
}

impl<W> Wtr<W>
where
    W: FnMut(&str, Option<&str>) -> Result<(), io::Error>,
{
    pub fn key2wtr(&mut self, t: &ReadOnlyTable<&str, &str>, key: &str) -> Result<(), io::Error> {
        let o: Option<_> = t.get(key).map_err(io::Error::other)?;
        let Some(guard) = o else {
            (self.wtr)(key, None)?;
            return Ok(());
        };

        let val: &str = guard.value();
        (self.wtr)(key, Some(val))
    }

    pub fn keys2wtr<I>(&mut self, t: &ReadOnlyTable<&str, &str>, keys: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<String, io::Error>>,
    {
        for rkey in keys {
            let key: String = rkey?;
            self.key2wtr(t, &key)?;
        }
        Ok(())
    }
}

pub fn wtr2json_wtr<W>(mut wtr: W) -> impl FnMut(&str, Option<&str>) -> Result<(), io::Error>
where
    W: Write,
{
    move |key: &str, val: Option<&str>| {
        let s = &[Some(key), val];
        serde_json::to_writer(&mut wtr, s)?;
        writeln!(wtr)?;
        Ok(())
    }
}

pub fn brdr2keys<R>(brdr: R) -> impl Iterator<Item = Result<String, io::Error>>
where
    R: BufRead,
{
    brdr.lines()
}

pub fn path2db2keys2vals2wtr<P, I, W>(
    path2redb: P,
    keys: I,
    wtr: W,
    tdef: TableDefinition<&str, &str>,
) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    I: Iterator<Item = Result<String, io::Error>>,
    W: FnMut(&str, Option<&str>) -> Result<(), io::Error>,
{
    let rod: ReadOnlyDatabase = ReadOnlyDatabase::open(path2redb).map_err(io::Error::other)?;
    let rtx: ReadTransaction = rod.begin_read().map_err(io::Error::other)?;
    let tab: ReadOnlyTable<&str, &str> = rtx.open_table(tdef).map_err(io::Error::other)?;
    Wtr { wtr }.keys2wtr(&tab, keys)?;
    Ok(())
}

pub fn path2db2keys2vals2iowtr_json<P, I, W>(
    path2redb: P,
    keys: I,
    mut wtr: W,
    tname: &str,
) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    I: Iterator<Item = Result<String, io::Error>>,
    W: Write,
{
    let jwtr = wtr2json_wtr(&mut wtr);
    let tdef: TableDefinition<&str, &str> = TableDefinition::new(tname);
    path2db2keys2vals2wtr(path2redb, keys, jwtr, tdef)?;
    wtr.flush()
}

pub fn path2db2stdin2keys2vals2stdout_json<P>(path2redb: P, tname: &str) -> Result<(), io::Error>
where
    P: AsRef<Path>,
{
    if tname.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty table name got",
        ));
    }

    let keys = brdr2keys(io::stdin().lock());
    let o = io::stdout();
    let mut ol = o.lock();
    path2db2keys2vals2iowtr_json(path2redb, keys, BufWriter::new(&mut ol), tname)?;
    ol.flush()
}

pub fn tabname2path2db2keys2vals_stdio(
    tname: String,
) -> impl FnMut(String) -> Result<(), io::Error> {
    move |path2redb: String| path2db2stdin2keys2vals2stdout_json(path2redb, &tname)
}

pub const TABLE_NAME_DEFAULT: &str = "default_redb_table";

pub fn tabname2path2db2keys2vals_stdio_default() -> impl FnMut(String) -> Result<(), io::Error> {
    tabname2path2db2keys2vals_stdio(TABLE_NAME_DEFAULT.into())
}
