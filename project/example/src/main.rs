use std::io::{Read, Seek, SeekFrom, Write};

use vfs_lib::*;

fn main() -> Result<(), Error> {
    std::fs::remove_file("realfile.vfs").unwrap_or_default();
    let vfs = Vfs::open("realfile.vfs");

    let mut f1 = vfs.create("file1.txt")?;
    vfs.create_dir("first_dir")?;

    vfs.create_dir("first_dir/second_dir")?;
    let mut f2 = vfs.create("first_dir/file2.txt")?;

    vfs.create("first_dir/second_dir/file😀.txt")?;
    vfs.create("first_dir/second_dir/file😮.txt")?;
    vfs.create("first_dir/second_dir/file😆.txt")?;

    vfs.print_tree();

    vfs.delete("first_dir/second_dir")?;

    println!();
    vfs.print_tree();

    f1.write_all("We 💚 Rust".as_bytes())?;
    f2.write_all("Hello Wolrd! 🖐".as_bytes())?;

    f1.seek(SeekFrom::Start(0))?;
    let mut text = String::new();
    f1.read_to_string(&mut text)?;
    println!("\n{}", text);

    f2.seek(SeekFrom::Start(0))?;
    let mut text = String::new();
    f2.read_to_string(&mut text)?;

    println!("\n{}", text);

    // let mut f = vfs.create("file")?;
    // vfs.delete("file");
    // let mut f2 = vfs.create("file2")?;
    // f.write_all(b"Hello World")?;

    Ok(())
}
