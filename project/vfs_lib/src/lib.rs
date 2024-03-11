// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// tests, deletion of directories and files

#[cfg(test)]
mod tests {
    use std::{fs::remove_file, str::from_utf8, thread::sleep};

    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }

    #[test]
    fn make_root_should_not_panic() {
        VfsInternal::make_root();
    }

    #[test]
    fn file_name_too_big_error() {
        let file = FileStruct {
            is_root: false,
            is_active: true,
            is_directory: true,
            name: "f".repeat(FILE_NAME_SIZE),
            contents: 0,
            next: 0,
            size: 0,
            creation_time: 0,
            last_write_time: 0,
        };
        let temp: Result<FileBytes, Error> = file.try_into();
        temp.unwrap();

        let file = FileStruct {
            is_root: false,
            is_active: true,
            is_directory: true,
            name: "f".repeat(FILE_NAME_SIZE + 1),
            contents: 0,
            next: 0,
            size: 0,
            creation_time: 0,
            last_write_time: 0,
        };
        let temp: Result<FileBytes, Error> = file.try_into();
        assert_eq!(temp.unwrap_err(), Error::FileNameTooBig);
    }

    #[test]
    fn bytes_to_struct_consistency() {
        let dummy = FileStruct {
            is_root: true,
            is_active: true,
            is_directory: true,
            name: "dummy".to_owned(),
            contents: 12,
            next: 13,
            size: 14,
            creation_time: 15,
            last_write_time: 16,
        };
        let temp: Result<FileBytes, Error> = dummy.clone().try_into();
        assert_eq!(dummy, FileStruct::from(temp.unwrap()))
    }

    #[test]
    fn tree_structure() {
        remove_file("test_tree.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_tree.vfs");

        vfs.create("file1.txt").unwrap();
        vfs.create("file2.txt").unwrap();
        vfs.create_dir("first_dir").unwrap();

        vfs.create_dir("first_dir/second_dir").unwrap();
        vfs.create("first_dir/file3.txt").unwrap();
        vfs.create("first_dir/file4.txt").unwrap();

        vfs.create_dir("first_dir/second_dir/third_dir").unwrap();
        vfs.create("first_dir/second_dir/file5.txt").unwrap();
        vfs.create("first_dir/second_dir/file6.txt").unwrap();

        vfs.print_tree();
    }

    #[test]
    fn test_create_file_name_size_err() {
        remove_file("create_file_name_size_err.vfs").unwrap_or_default();
        let vfs = Vfs::open("create_file_name_size_err.vfs");

        assert_eq!(vfs.create(&"f".repeat(FILE_NAME_SIZE)).is_ok(), true);
        assert_eq!(
            vfs.create(&"f".repeat(FILE_NAME_SIZE + 1)).unwrap_err(),
            Error::FileNameTooBig
        );
    }

    #[test]
    fn read_write_to_single_file() {
        remove_file("test_read_write.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_read_write.vfs");

        let mut f1 = vfs.create("file1.txt").unwrap();

        f1.write_all(b"Hello World!").unwrap();
        f1.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        f1.read_to_end(&mut bytes).unwrap();
        let text = from_utf8(&bytes).unwrap();

        assert_eq!(text, "Hello World!");
    }

    #[test]
    fn read_write_to_multiple_files() {
        remove_file("test_read_write_2.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_read_write_2.vfs");

        let mut f1 = vfs.create("file1.txt").unwrap();
        let mut f2 = vfs.create("file2.txt").unwrap();

        f1.write_all(b"Hello World!").unwrap();
        f2.write_all(b"Hello World! again").unwrap();

        let mut bytes;

        bytes = Vec::new();
        f1.seek(SeekFrom::Start(0)).unwrap();
        f1.read_to_end(&mut bytes).unwrap();
        let text1 = from_utf8(&bytes).unwrap().to_owned();

        bytes = Vec::new();
        f2.seek(SeekFrom::Start(0)).unwrap();
        f2.read_to_end(&mut bytes).unwrap();
        let text2 = from_utf8(&bytes).unwrap().to_owned();

        assert_eq!(text1, "Hello World!");
        assert_eq!(text2, "Hello World! again");
    }

    #[test]
    fn write_overflow_to_file() {
        remove_file("test_write_overflow.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_write_overflow.vfs");

        let mut f1 = vfs.create("file1.txt").unwrap();
        let mut f2 = vfs.create("file2.txt").unwrap();

        f1.write_all(b"Hello World!").unwrap();
        f2.write_all(b"Hello World! again").unwrap();

        f1.write_all(&"f".repeat(DEFAULT_PAGE_SIZE as usize).into_bytes())
            .unwrap(); //vfs.page_size as usize

        let f1_metadata = f1.get_metadata();
        let f2_metadata = f2.get_metadata();

        assert_eq!(f1_metadata.contents, 6);
        assert_eq!(
            f1_metadata.size as usize,
            b"Hello World!".len() + DEFAULT_PAGE_SIZE as usize
        );
        assert_eq!(f2_metadata.contents, 5);
        assert_eq!(f2_metadata.size as usize, b"Hello World! again".len());
    }

    #[test]
    fn test_file_table_overflow() {
        remove_file("test_file_table_overflow.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_file_table_overflow.vfs");
        let page_size = vfs.internal.borrow().page_size;
        for i in 0..(page_size / FILE_STRUCT_SIZE as u64 + 1) {
            vfs.create(&format!("file{}.txt", i)).unwrap();
        }
        let internal = vfs.internal.borrow();

        assert_eq!(internal.file_table_page, DEFAULT_FILE_TABLE_PAGE);
        assert_eq!(internal.file_table_size, 2);
    }

    #[test]
    fn test_file_table_overflow_write_to_file() {
        remove_file("test_big_table_write_file.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_big_table_write_file.vfs");
        let page_size = vfs.internal.borrow().page_size;
        for i in 0..(page_size / FILE_STRUCT_SIZE as u64 + 1) {
            vfs.create(&format!("file{}.txt", i)).unwrap();
        }

        let mut f = vfs.open_file("file2.txt").unwrap();

        f.write_all(b"Hello World!").unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        assert_eq!(text, "Hello World!")
    }

    #[test]
    fn test_cannot_use_nonexistent_file_system() {
        let mut f;
        let mut dir;

        {
            remove_file("nonexistent_file_system.vfs").unwrap_or_default();
            let vfs = Vfs::open("nonexistent_file_system.vfs");

            f = vfs.create("file.txt").unwrap();
            vfs.create_dir("test_dir").unwrap();
            vfs.create("test_dir/file2.txt").unwrap();
            vfs.create("test_dir/file3.txt").unwrap();
            dir = vfs.read_dir("test_dir").unwrap();
        }

        assert_eq!(
            f.write_all(b"Hello World!").unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(
            f.seek(SeekFrom::Start(0)).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        let mut text = String::new();
        assert_eq!(
            f.read_to_string(&mut text).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(f.metadata().unwrap_err(), Error::FileNotFound);
        assert_eq!(dir.next().is_none(), true);
    }

    #[test]
    fn test_cannot_have_dulicate_names() {
        remove_file("duplicate_names.vfs").unwrap_or_default();
        let vfs = Vfs::open("duplicate_names.vfs");

        vfs.create("file1").unwrap();
        assert_eq!(vfs.create("file1").unwrap_err(), Error::NameAlreadyInUse);
        assert_eq!(
            vfs.create_dir("file1").unwrap_err(),
            Error::NameAlreadyInUse
        );

        vfs.create_dir("dir1").unwrap();
        assert_eq!(vfs.create("dir1").unwrap_err(), Error::NameAlreadyInUse);
        assert_eq!(vfs.create_dir("dir1").unwrap_err(), Error::NameAlreadyInUse);
    }

    #[test]
    fn test_cannot_create_in_nonexistent_directory() {
        remove_file("nonexistent_directory.vfs").unwrap_or_default();
        let vfs = Vfs::open("nonexistent_directory.vfs");

        vfs.create_dir("dir1").unwrap();
        assert_eq!(
            vfs.create("dir1/dir2/file1").unwrap_err(),
            Error::DirectoryNotFound
        );
        assert_eq!(
            vfs.create_dir("dir1/dir2/dir3").unwrap_err(),
            Error::DirectoryNotFound
        );
    }

    #[test]
    fn test_cannot_create_files_when_full() {
        remove_file("test_file_table_overflow_when_full.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_file_table_overflow_when_full.vfs");
        let page_size = vfs.internal.borrow().page_size;
        for i in 0..(page_size / FILE_STRUCT_SIZE as u64 - 1) {
            vfs.create(&format!("file{}.txt", i)).unwrap();
        }

        {
            let internal = vfs.internal.borrow();
            internal
                .allocate_page_range(0..(8 * internal.page_size), true)
                .unwrap();
        }

        assert_eq!(
            vfs.create("another_file").unwrap_err(),
            Error::IO(std::io::ErrorKind::OutOfMemory)
        );
        assert_eq!(
            vfs.create_dir("another_dir").unwrap_err(),
            Error::IO(std::io::ErrorKind::OutOfMemory)
        );
    }

    #[test]
    fn write_overflow_to_file_when_full() {
        remove_file("test_write_overflow_when_full.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_write_overflow_when_full.vfs");

        let mut f1 = vfs.create("file1.txt").unwrap();
        let mut f2 = vfs.create("file2.txt").unwrap();

        f1.write_all(b"Hello World!").unwrap();
        f2.write_all(b"Hello World! again").unwrap();

        {
            let internal = vfs.internal.borrow();
            internal
                .allocate_page_range(0..(8 * internal.page_size), true)
                .unwrap();
        }

        assert_eq!(
            f1.write_all(&"f".repeat(DEFAULT_PAGE_SIZE as usize).into_bytes())
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::OutOfMemory
        );

        assert_eq!(f1.metadata().unwrap().size, b"Hello World!".len() as u64)
    }

    #[test]
    fn test_cannot_open_dir_as_file() {
        remove_file("dir_file_confusion.vfs").unwrap_or_default();
        let vfs = Vfs::open("dir_file_confusion.vfs");

        vfs.create("file").unwrap();
        vfs.create_dir("dir").unwrap();

        assert_eq!(vfs.open_file("dir").unwrap_err(), Error::FileNotFound);
        assert_eq!(vfs.read_dir("file").unwrap_err(), Error::DirectoryNotFound);
    }

    #[test]
    fn test_data_persistency() {
        {
            remove_file("data_persistency.vfs").unwrap_or_default();
            let vfs = Vfs::open("data_persistency.vfs");

            vfs.create_dir("dir1").unwrap();
            vfs.create_dir("dir1/dir2").unwrap();
            let mut f = vfs.create("dir1/dir2/file").unwrap();
            f.write_all(b"Hello World!").unwrap();
        }

        let vfs = Vfs::open("data_persistency.vfs");
        let mut f = vfs.open_file("dir1/dir2/file").unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        assert_eq!(text, "Hello World!");
    }

    #[test]
    fn test_last_write_time() {
        remove_file("last_write_time.vfs").unwrap_or_default();
        let vfs = Vfs::open("last_write_time.vfs");
        let mut f = vfs.create("file1").unwrap();
        sleep(std::time::Duration::new(1, 0));
        f.write_all(b"lorem ipsum").unwrap();
        let metadata = f.metadata().unwrap();
        assert!(
            metadata.creation_time + 1 <= metadata.last_write_time
                && metadata.last_write_time <= metadata.creation_time + 2
        );
    }

    #[test]
    fn test_cannot_open_nonexistent_file() {
        remove_file("open_nonexistent_file.vfs").unwrap_or_default();
        let vfs = Vfs::open("open_nonexistent_file.vfs");
        vfs.create("file1").unwrap();
        vfs.create_dir("dir1").unwrap();

        assert_eq!(vfs.open_file("file2").unwrap_err(), Error::FileNotFound);
        assert_eq!(vfs.read_dir("dir2").unwrap_err(), Error::DirectoryNotFound);
    }

    #[test]
    fn test_delete_file_structure() {
        remove_file("open_deleted_file.vfs").unwrap_or_default();
        let vfs = Vfs::open("open_deleted_file.vfs");
        vfs.create("file1").unwrap();
        vfs.create_dir("dir1").unwrap();

        vfs.delete("file1").unwrap();
        vfs.delete("dir1").unwrap();

        assert_eq!(vfs.open_file("file1").unwrap_err(), Error::FileNotFound);
        assert_eq!(vfs.read_dir("dir1").unwrap_err(), Error::DirectoryNotFound);

        assert_eq!(vfs.delete("file1").unwrap_err(), Error::FileNotFound);
        assert_eq!(vfs.delete("dir1").unwrap_err(), Error::FileNotFound);

        vfs.create_dir("dir1").unwrap();
        vfs.create("other_file").unwrap();
        vfs.create("dir1/file4").unwrap();
        vfs.create_dir("dir1/dir2").unwrap();
        vfs.create("dir1/dir2/file1").unwrap();
        vfs.create_dir("dir1/dir2/dir3").unwrap();
        vfs.create("dir1/file5").unwrap();

        vfs.delete("dir1/dir2").unwrap();
        assert_eq!(
            vfs.open_file("dir1/dir2/file1").unwrap_err(),
            Error::FileNotFound
        );
        assert_eq!(
            vfs.read_dir("dir1/dir2/dir3").unwrap_err(),
            Error::DirectoryNotFound
        );
        assert_eq!(
            vfs.read_dir("dir1/dir2").unwrap_err(),
            Error::DirectoryNotFound
        );

        assert_eq!(vfs.read_dir("dir1").is_ok(), true);
        assert_eq!(vfs.open_file("other_file").is_ok(), true);
        assert_eq!(vfs.open_file("dir1/file4").is_ok(), true);
        assert_eq!(vfs.open_file("dir1/file5").is_ok(), true);
    }

    #[test]
    fn test_delete_file_contents() {
        remove_file("delete_file_contents.vfs").unwrap_or_default();
        let vfs = Vfs::open("delete_file_contents.vfs");
        {
            let mut f = vfs.create("file.txt").unwrap();
            f.write_all(
                &"c".repeat((3 * DEFAULT_PAGE_SIZE + DEFAULT_PAGE_SIZE / 2) as usize)
                    .as_bytes(),
            )
            .unwrap();
        }

        {
            let internal = vfs.internal.borrow_mut();
            assert_eq!(internal.is_page_allocated(4).unwrap(), true);
            assert_eq!(internal.is_page_allocated(5).unwrap(), true);
            assert_eq!(internal.is_page_allocated(6).unwrap(), true);
            assert_eq!(internal.is_page_allocated(7).unwrap(), true);
        }

        vfs.delete("file.txt").unwrap();

        {
            let internal = vfs.internal.borrow_mut();
            assert_eq!(internal.is_page_allocated(4).unwrap(), false);
            assert_eq!(internal.is_page_allocated(5).unwrap(), false);
            assert_eq!(internal.is_page_allocated(6).unwrap(), false);
            assert_eq!(internal.is_page_allocated(7).unwrap(), false);
        }
    }

    #[test]
    fn test_read_dir_deleted_files() {
        remove_file("test_read_dir_deleted_files.vfs").unwrap_or_default();
        let vfs = Vfs::open("test_read_dir_deleted_files.vfs");
        vfs.create_dir("dir").unwrap();
        vfs.create("dir/file1").unwrap();
        vfs.create("dir/file2").unwrap();
        vfs.create("dir/file3").unwrap();

        let dir = vfs.read_dir("dir").unwrap();

        vfs.delete("dir/file2").unwrap();

        let mut found1 = false;
        let mut found2 = false;
        let mut found3 = false;
        for entry in dir {
            if entry.is_ok() {
                let entry = entry.unwrap();
                if entry.get_path() == "dir/file1" {
                    found1 = true;
                }

                if entry.get_path() == "dir/file2" {
                    found2 = true;
                }

                if entry.get_path() == "dir/file3" {
                    found3 = true;
                }
            } else {
                assert_eq!(entry.unwrap_err(), Error::FileNotFound);
            }
        }

        assert_eq!(found1, true);
        assert_eq!(found2, false);
        assert_eq!(found3, true);
    }

    #[test]
    fn example() -> Result<(), Error> {
        remove_file("example.vfs").unwrap_or_default();
        let vfs = Vfs::open("example.vfs");

        vfs.create_dir("rs")?;
        {
            let mut f1 = vfs.create("rs/abc.txt")?;
            let mut f2 = vfs.create("rs/def.txt")?;

            f1.write_all(b"hello")?;
            f2.write_all(b"world")?;
        }

        let mut data = String::new();
        for entry in vfs.read_dir("rs")? {
            let entry = entry?;
            data.clear();

            let mut file = vfs.open_entry(entry)?;
            file.read_to_string(&mut data)?;

            print!("{}", data);
        }
        println!();
        Ok(())
    }
}

use std::cell::RefCell;
use std::fs::{self};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::time::SystemTime;

const DEFAULT_PAGE_SIZE: u64 = 4096;
const DEFAULT_ALLOCATION_TABLE_PAGE: u64 = 1;
const DEFAULT_SYSTEM_COMMIT_PAGE: u64 = 2;
const DEFAULT_FILE_TABLE_PAGE: u64 = 3;

pub struct Vfs {
    internal: Rc<RefCell<VfsInternal>>,
}

impl Vfs {
    pub fn open(filename: &str) -> Vfs {
        Vfs {
            internal: Rc::new(RefCell::new(VfsInternal::open(filename))),
        }
    }

    pub fn create_dir(&self, path: &str) -> Result<(), Error> {
        self.create_file_entity(path, true)?;
        Ok(())
    }

    pub fn create(&self, path: &str) -> Result<File, Error> {
        self.create_file_entity(path, false)
    }

    fn create_file_entity(&self, path: &str, is_directory: bool) -> Result<File, Error> {
        let mut path_componenets = path.split('/');

        let name = path_componenets.next_back().unwrap();

        if name.len() > FILE_NAME_SIZE {
            return Err(Error::FileNameTooBig);
        }

        let mut internal = self.internal.borrow_mut();

        let mut previous_pointer = 0;
        let mut child_entity = internal.get_file_struct_by_index(previous_pointer);
        let mut current_pointer = child_entity.contents;

        for path_dir in path_componenets {
            if path_dir.is_empty() {
                continue;
            }

            let mut found = false;

            while current_pointer != 0 {
                child_entity = internal.get_file_struct_by_index(current_pointer);

                if child_entity.name == path_dir && child_entity.is_active {
                    if child_entity.is_directory {
                        previous_pointer = current_pointer;
                        current_pointer = child_entity.contents;
                        found = true;
                        break;
                    } else {
                        return Err(Error::DirectoryNotFound);
                    }
                }

                current_pointer = child_entity.next;
            }

            if !found {
                return Err(Error::DirectoryNotFound);
            }
        }

        //previous_pointer contains parent directory of where we want to create the entity
        //current_pointer contains first entity in parent directory

        //search if it already exists or the last file in the folder if it doesn't

        let is_parent_dir_empty = current_pointer == 0;

        while current_pointer != 0 {
            child_entity = internal.get_file_struct_by_index(current_pointer);

            if child_entity.name == name && child_entity.is_active {
                return Err(Error::NameAlreadyInUse);
            }

            previous_pointer = current_pointer;
            current_pointer = child_entity.next;
        }

        let new_index = internal.find_inactive_file_slot();

        if new_index.is_none() {
            let number_of_pages_needed = internal.file_table_size + 1;
            let temp = internal.find_first_fitting_page_range(
                number_of_pages_needed,
                internal.file_table_page..(internal.file_table_page + internal.file_table_size),
            );
            if temp.is_none() {
                return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory).into());
            }
            let contents_location = temp.unwrap();

            //critical
            let mut modifications = Vec::new();

            modifications.push(Modification::AllcationTable(
                internal.file_table_page..(internal.file_table_page + internal.file_table_size),
                false,
            ));
            modifications.push(Modification::AllcationTable(
                contents_location..(contents_location + number_of_pages_needed),
                true,
            ));

            let mut last_page = internal.get_number_of_pages();
            if internal.file_table_page <= last_page
                && last_page < internal.file_table_page + internal.file_table_size
            {
                last_page = internal.file_table_page - 1;
            }
            if last_page < contents_location + number_of_pages_needed - 1 {
                last_page = contents_location + number_of_pages_needed - 1;
            }
            let vfs_page_total = last_page + 1;

            {
                let mut physical_file = internal.physical_file.borrow_mut();
                physical_file.set_len(vfs_page_total * internal.page_size)?;

                let mut contents_buffer: Vec<u8> = Vec::new();
                contents_buffer.resize((internal.file_table_size * internal.page_size) as usize, 0);
                physical_file
                    .seek(SeekFrom::Start(
                        internal.file_table_page * internal.page_size,
                    ))
                    .unwrap();
                physical_file.read_exact(&mut contents_buffer).unwrap();

                physical_file
                    .seek(SeekFrom::Start(contents_location * internal.page_size))
                    .unwrap();
                physical_file.write_all(&contents_buffer).unwrap();
            }

            //critical

            internal.file_table_page = contents_location;
            internal.file_table_size += 1;

            modifications.push(Modification::SystemHeader(
                internal.file_table_page,
                internal.file_table_size,
            ));

            internal.schedule_commit(modifications);
            internal.resolve_commit();
        }

        let new_index = internal.find_inactive_file_slot().unwrap();

        let time = VfsInternal::get_system_time();

        let new_entity = FileStruct {
            is_root: false,
            is_active: true,
            is_directory,
            name: name.to_owned(),
            contents: 0,
            next: 0,
            size: 0,
            creation_time: time,
            last_write_time: time,
        };

        if is_parent_dir_empty {
            child_entity.contents = new_index;
        } else {
            child_entity.next = new_index;
        }

        let modifications = vec![
            Modification::FileTable(new_index, new_entity),
            Modification::FileTable(previous_pointer, child_entity),
        ];

        internal.schedule_commit(modifications);
        internal.resolve_commit();

        Ok(File {
            internal: Rc::downgrade(&self.internal),
            file_index: new_index,
            cursor: 0,
            path: path.to_owned(),
        })
    }

    pub fn print_tree(&self) {
        let internal = self.internal.borrow_mut();
        let root_contents = internal.get_file_struct_by_index(0).contents;
        internal.print_tree_recursive(root_contents, 0);
    }

    pub fn read_dir(&self, path: &str) -> Result<DirIterator, Error> {
        let internal = self.internal.borrow_mut();
        let temp = internal.get_file_struct_by_path(path);

        if temp.is_err() {
            return Err(Error::DirectoryNotFound);
        }

        let dir = temp.unwrap().1;

        let mut entry_names = Vec::new();

        let mut pointer = dir.contents;
        while pointer != 0 {
            let entry = internal.get_file_struct_by_index(pointer);
            entry_names.push(entry.name);
            pointer = entry.next;
        }

        if dir.is_directory {
            Ok(DirIterator {
                cursor: 0,
                internal: Rc::downgrade(&self.internal),
                path: path.to_owned(),
                entry_names,
            })
        } else {
            Err(Error::DirectoryNotFound)
        }
    }

    pub fn open_entry(&self, file: DirEntry) -> Result<File, Error> {
        self.open_file(&file.path)
    }

    pub fn open_file(&self, path: &str) -> Result<File, Error> {
        let internal = self.internal.borrow_mut();
        let temp = internal.get_file_struct_by_path(path);

        if temp.is_err() {
            return Err(Error::FileNotFound);
        }

        let (index, file) = temp.unwrap();

        if !file.is_directory {
            Ok(File {
                file_index: index,
                cursor: 0,
                internal: Rc::downgrade(&self.internal),
                path: path.to_owned(),
            })
        } else {
            Err(Error::FileNotFound)
        }
    }

    pub fn delete(&self, path: &str) -> Result<(), Error> {
        if path.is_empty() {
            return Ok(());
        }
        let mut path_componenets = path.split('/');

        let name = path_componenets.next_back().unwrap();

        let internal = self.internal.borrow_mut();

        let mut previous_pointer = 0;
        let mut child_entity = internal.get_file_struct_by_index(previous_pointer);
        let mut current_pointer = child_entity.contents;

        for path_dir in path_componenets {
            if path_dir.is_empty() {
                continue;
            }

            let mut found = false;

            while current_pointer != 0 {
                child_entity = internal.get_file_struct_by_index(current_pointer);

                if child_entity.name == path_dir && child_entity.is_active {
                    if child_entity.is_directory {
                        previous_pointer = current_pointer;
                        current_pointer = child_entity.contents;
                        found = true;
                        break;
                    } else {
                        return Err(Error::DirectoryNotFound);
                    }
                }

                current_pointer = child_entity.next;
            }

            if !found {
                return Err(Error::DirectoryNotFound);
            }
        }

        //previous_pointer contains parent directory of where we want to create the entity
        //current_pointer contains first entity in parent directory

        let previous_pointer_copy = previous_pointer;
        let mut found = false;

        while current_pointer != 0 {
            child_entity = internal.get_file_struct_by_index(current_pointer);

            if child_entity.name == name && child_entity.is_active {
                found = true;
                break;
            }

            previous_pointer = current_pointer;
            current_pointer = child_entity.next;
        }

        if !found {
            return Err(Error::FileNotFound);
        }

        let is_previous_pointer_parent_dir = previous_pointer == previous_pointer_copy;

        internal.delete_recursive(
            previous_pointer,
            current_pointer,
            is_previous_pointer_parent_dir,
        );

        Ok(())
    }
}
struct VfsInternal {
    physical_file: RefCell<fs::File>,
    page_size: u64,
    file_table_page: u64,
    file_table_size: u64,
}

impl VfsInternal {
    fn open(filename: &str) -> VfsInternal {
        let new_vfs;

        let already_exists = Path::new(filename).exists();

        if !already_exists {
            let file = fs::File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(filename)
                .unwrap();
            new_vfs = VfsInternal::default(file);

            let file_table_index = new_vfs.get_file_table_index();
            let allocation_table_index = DEFAULT_ALLOCATION_TABLE_PAGE * new_vfs.page_size;
            let mut file = new_vfs.physical_file.borrow_mut();

            //make room for header + allocation table + system commit + file table
            file.set_len(4 * new_vfs.page_size).unwrap();

            //leave space for nullptr, then write page size
            file.seek(SeekFrom::Start(size_of::<u64>() as u64)).unwrap();
            file.write_all(&new_vfs.page_size.to_le_bytes()).unwrap();

            //write page number of file table
            file.write_all(&new_vfs.file_table_page.to_le_bytes())
                .unwrap();
            file.write_all(&new_vfs.file_table_size.to_le_bytes())
                .unwrap();

            //allocate system pages
            file.seek(SeekFrom::Start(allocation_table_index)).unwrap();
            file.write_all(&[0b0000_1111]).unwrap();

            //write root in file table page
            file.seek(SeekFrom::Start(file_table_index)).unwrap();
            file.write_all(&Self::make_root()).unwrap();
        } else {
            let mut file = fs::File::options()
                .read(true)
                .write(true)
                .open(filename)
                .unwrap();
            let mut int_buffer = [0u8; size_of::<u64>()];

            //read page size
            file.seek(SeekFrom::Start(size_of::<u64>() as u64)).unwrap();
            file.read_exact(&mut int_buffer).unwrap();
            let page_size = u64::from_le_bytes(int_buffer);

            //read file table page
            file.read_exact(&mut int_buffer).unwrap();
            let file_table_page = u64::from_le_bytes(int_buffer);

            //read file table size
            file.read_exact(&mut int_buffer).unwrap();
            let file_table_size = u64::from_le_bytes(int_buffer);

            new_vfs = VfsInternal {
                physical_file: RefCell::new(file),
                page_size,
                file_table_page,
                file_table_size,
            };

            new_vfs.resolve_commit();
        }

        new_vfs
    }

    fn default(physical_file: fs::File) -> VfsInternal {
        VfsInternal {
            physical_file: RefCell::new(physical_file),
            page_size: DEFAULT_PAGE_SIZE,
            file_table_page: DEFAULT_FILE_TABLE_PAGE,
            file_table_size: 1,
        }
    }

    fn make_root() -> FileBytes {
        let time = Self::get_system_time();
        let root = FileStruct {
            is_root: true,
            is_active: true,
            is_directory: true,
            name: "root".to_owned(),
            contents: 0,
            next: 0,
            size: 0,
            creation_time: time,
            last_write_time: time,
        };
        root.try_into().unwrap()
    }

    fn get_file_struct_by_index(&self, index: u64) -> FileStruct {
        if index + FILE_STRUCT_SIZE as u64 >= self.file_table_size * self.page_size {
            return [0; FILE_STRUCT_SIZE].into();
        }
        let mut physical_file = self.physical_file.borrow_mut();
        physical_file
            .seek(SeekFrom::Start(
                self.file_table_page * self.page_size + index,
            ))
            .unwrap();
        let mut file_struct_buffer = [0u8; FILE_STRUCT_SIZE];
        physical_file.read_exact(&mut file_struct_buffer).unwrap();
        FileStruct::from(file_struct_buffer)
    }

    fn get_file_struct_by_path(&self, path: &str) -> Result<(u64, FileStruct), Error> {
        let mut path_componenets = path.split('/');

        let name = path_componenets.next_back().unwrap();

        let mut child_entity = self.get_file_struct_by_index(0);
        let mut current_pointer = child_entity.contents;

        for path_dir in path_componenets {
            if path_dir.is_empty() {
                continue;
            }

            let mut found = false;

            while current_pointer != 0 {
                child_entity = self.get_file_struct_by_index(current_pointer);

                if child_entity.name == path_dir && child_entity.is_active {
                    if child_entity.is_directory {
                        current_pointer = child_entity.contents;
                        found = true;
                        break;
                    } else {
                        return Err(Error::DirectoryNotFound);
                    }
                }

                current_pointer = child_entity.next;
            }

            if !found {
                return Err(Error::DirectoryNotFound);
            }
        }

        while current_pointer != 0 {
            child_entity = self.get_file_struct_by_index(current_pointer);

            if child_entity.name == name && child_entity.is_active {
                return Ok((current_pointer, child_entity));
            }

            current_pointer = child_entity.next;
        }

        Err(Error::FileNotFound)
    }

    fn find_inactive_file_slot(&self) -> Option<u64> {
        let mut flags = [0u8; 1];
        let mut index = self.get_file_table_index();

        let mut physical_file = self.physical_file.borrow_mut();

        physical_file.seek(SeekFrom::Start(index)).unwrap();
        physical_file.read_exact(&mut flags).unwrap();

        while flags[0] & 0b10 != 0 {
            //is active
            index = physical_file
                .seek(SeekFrom::Current(FILE_STRUCT_SIZE as i64 - 1))
                .unwrap();
            index -= self.get_file_table_index();

            if index + FILE_STRUCT_SIZE as u64 >= self.file_table_size * self.page_size {
                return None;
            }

            physical_file.read_exact(&mut flags).unwrap();
        }

        Some(index)
    }

    fn get_system_time() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            // .as_millis() as u64
            .as_secs()
    }

    fn get_file_table_index(&self) -> u64 {
        self.page_size * self.file_table_page
    }

    fn print_tree_recursive(&self, from_index: u64, depth: u64) {
        let mut pointer = from_index;
        let mut entity: FileStruct;
        while pointer != 0 {
            entity = self.get_file_struct_by_index(pointer);

            if entity.is_active {
                print!("{}", "---".repeat(depth as usize));
                println!(
                    "{} : {}",
                    entity.name,
                    if entity.is_directory {
                        "directory"
                    } else {
                        "file"
                    }
                );

                if entity.is_directory {
                    self.print_tree_recursive(entity.contents, depth + 1);
                }
            }
            pointer = entity.next;
        }
    }

    fn allocate_page(&self, page_number: u64, is_allocated: bool) -> Result<(), Error> {
        if page_number >= self.page_size * 8 {
            return Err(Error::PageNumberTooBig);
        }

        let mut physical_file = self.physical_file.borrow_mut();

        let byte_location = page_number / 8;
        let bit_location = page_number % 8;

        physical_file
            .seek(SeekFrom::Start(
                self.page_size * DEFAULT_ALLOCATION_TABLE_PAGE + byte_location,
            ))
            .unwrap();
        let mut byte = [0; 1];
        physical_file.read_exact(&mut byte).unwrap();
        byte[0] = if is_allocated {
            byte[0] | 1 << bit_location
        } else {
            byte[0] & !(1 << bit_location)
        };
        physical_file.seek(SeekFrom::Current(-1)).unwrap();
        physical_file.write_all(&byte).unwrap();

        //println!("{:<08b} {}", byte[0], page_number);

        Ok(())
    }

    fn allocate_page_range(
        &self,
        page_range: std::ops::Range<u64>,
        is_allocated: bool,
    ) -> Result<(), Error> {
        if page_range.end > self.page_size * 8 {
            return Err(Error::PageNumberTooBig);
        }

        if page_range.is_empty() {
            return Ok(());
        }

        for page_number in page_range {
            self.allocate_page(page_number, is_allocated)?;
        }

        Ok(())
    }

    fn is_page_allocated(&self, page_number: u64) -> Result<bool, Error> {
        if page_number >= self.page_size {
            return Err(Error::PageNumberTooBig);
        }

        let mut physical_file = self.physical_file.borrow_mut();

        let byte_location = page_number / 8;
        let bit_location = page_number % 8;

        physical_file
            .seek(SeekFrom::Start(
                self.page_size * DEFAULT_ALLOCATION_TABLE_PAGE + byte_location,
            ))
            .unwrap();
        let mut byte = [0; 1];
        physical_file.read_exact(&mut byte).unwrap();

        //println!("{} -> ({}, {}) -> {:b} ", page_number, byte_location, bit_location, byte[0]);

        Ok(byte[0] & 1 << bit_location != 0)
    }

    fn find_first_fitting_page_range(
        &self,
        number_of_pages_needed: u64,
        reallocating_pages: std::ops::Range<u64>,
    ) -> Option<u64> {
        let mut i = 0;
        let mut is_potential_range = false;
        let mut potential_range_position = 0;
        let mut potential_range_length = 0;
        while i < self.page_size {
            let is_current_page_allocated = self.is_page_allocated(i).unwrap()
                && !(reallocating_pages.start <= i && i < reallocating_pages.end);
            if is_potential_range {
                if is_current_page_allocated {
                    is_potential_range = false;
                } else {
                    potential_range_length += 1;
                }
            } else if !is_current_page_allocated {
                is_potential_range = true;
                potential_range_length = 1;
                potential_range_position = i;
            }

            //println!("pos: {} -> {}, {}", i, is_potential_range, potential_range_length);
            if is_potential_range && potential_range_length == number_of_pages_needed {
                return Some(potential_range_position);
            }

            i += 1;
        }
        None
    }

    fn get_number_of_pages(&self) -> u64 {
        let mut last_allocated_page = 0;
        for i in 0..self.page_size {
            if self.is_page_allocated(i).unwrap() {
                last_allocated_page = i;
            }
        }
        last_allocated_page + 1
    }

    fn update_file_by_index(&self, index: u64, file: FileStruct) {
        let mut physical_file = self.physical_file.borrow_mut();
        physical_file
            .seek(SeekFrom::Start(self.get_file_table_index() + index))
            .unwrap();
        let bytes: FileBytes = file.try_into().unwrap();
        physical_file.write_all(&bytes).unwrap();
    }

    fn schedule_commit(&self, modifications: Vec<Modification>) {
        let mut bytes: Vec<u8> = Vec::new();
        let count = modifications.len() as u8;
        for modification in modifications {
            match modification {
                Modification::FileTable(index, file) => {
                    bytes.push(1);
                    bytes.append(&mut index.to_le_bytes().to_vec());
                    let file_bytes: FileBytes = file.try_into().unwrap();
                    bytes.append(&mut file_bytes.to_vec());
                }
                Modification::AllcationTable(range, is_allocated) => {
                    bytes.push(2);
                    bytes.append(&mut range.start.to_le_bytes().to_vec());
                    bytes.append(&mut range.end.to_le_bytes().to_vec());
                    bytes.push(is_allocated as u8);
                }
                Modification::SystemHeader(file_table_page, file_table_size) => {
                    bytes.push(0);
                    bytes.append(&mut file_table_page.to_le_bytes().to_vec());
                    bytes.append(&mut file_table_size.to_le_bytes().to_vec());
                }
            }
        }

        let mut physical_file = self.physical_file.borrow_mut();
        physical_file
            .seek(SeekFrom::Start(
                DEFAULT_SYSTEM_COMMIT_PAGE * self.page_size + 1,
            ))
            .unwrap();
        physical_file.write_all(&bytes).unwrap();
        physical_file.flush().unwrap();
        physical_file
            .seek(SeekFrom::Start(DEFAULT_SYSTEM_COMMIT_PAGE * self.page_size))
            .unwrap();
        physical_file.write_all(&[count]).unwrap();
        physical_file.flush().unwrap();
    }

    fn resolve_commit(&self) {
        let mut count = [0; 1];

        {
            let mut physical_file = self.physical_file.borrow_mut();
            physical_file
                .seek(SeekFrom::Start(DEFAULT_SYSTEM_COMMIT_PAGE * self.page_size))
                .unwrap();
            physical_file.read_exact(&mut count).unwrap();
        }

        let mut modifications = Vec::new();

        for _ in 0..count[0] {
            let mut type_byte = [0; 1];

            {
                let mut physical_file = self.physical_file.borrow_mut();
                physical_file.read_exact(&mut type_byte).unwrap();
            }

            match type_byte[0] {
                0 => {
                    let mut page_bytes = 0u64.to_le_bytes();
                    let mut size_bytes = 0u64.to_le_bytes();
                    {
                        let mut physical_file = self.physical_file.borrow_mut();
                        physical_file.read_exact(&mut page_bytes).unwrap();
                        physical_file.read_exact(&mut size_bytes).unwrap();
                    }
                    modifications.push(Modification::SystemHeader(
                        u64::from_le_bytes(page_bytes),
                        u64::from_le_bytes(size_bytes),
                    ));
                }
                1 => {
                    let mut index_bytes = 0u64.to_le_bytes();
                    let mut file_bytes: FileBytes = [0; FILE_STRUCT_SIZE];
                    {
                        let mut physical_file = self.physical_file.borrow_mut();
                        physical_file.read_exact(&mut index_bytes).unwrap();
                        physical_file.read_exact(&mut file_bytes).unwrap();
                    }
                    modifications.push(Modification::FileTable(
                        u64::from_le_bytes(index_bytes),
                        file_bytes.into(),
                    ));
                    //self.update_file_by_index(u64::from_le_bytes(index_bytes), file_bytes.into())
                }
                2 => {
                    let mut start_bytes = 0u64.to_le_bytes();
                    let mut end_bytes = 0u64.to_le_bytes();
                    let mut is_allocated_byte = [0; 1];
                    {
                        let mut physical_file = self.physical_file.borrow_mut();
                        physical_file.read_exact(&mut start_bytes).unwrap();
                        physical_file.read_exact(&mut end_bytes).unwrap();
                        physical_file.read_exact(&mut is_allocated_byte).unwrap();
                    }

                    modifications.push(Modification::AllcationTable(
                        u64::from_le_bytes(start_bytes)..u64::from_le_bytes(end_bytes),
                        is_allocated_byte[0] != 0,
                    ));
                    //self.allocate_page_range(u64::from_le_bytes(start_bytes)..u64::from_le_bytes(end_bytes), is_allocated_byte[0] != 0).unwrap();
                }
                _ => {}
            }
        }

        for modification in modifications {
            match modification {
                Modification::FileTable(index, file) => self.update_file_by_index(index, file),
                Modification::AllcationTable(range, is_allocated) => {
                    self.allocate_page_range(range, is_allocated).unwrap()
                }
                Modification::SystemHeader(file_table_page, file_table_size) => {
                    self.update_header(file_table_page, file_table_size)
                }
            }
        }

        {
            let number_of_pages = self.get_number_of_pages();
            let mut physical_file = self.physical_file.borrow_mut();
            physical_file
                .set_len(number_of_pages * self.page_size)
                .unwrap();
            physical_file
                .seek(SeekFrom::Start(DEFAULT_SYSTEM_COMMIT_PAGE * self.page_size))
                .unwrap();
            physical_file.write_all(&[0]).unwrap();
            physical_file.flush().unwrap();
        }
    }

    fn update_header(&self, file_table_page: u64, file_table_size: u64) {
        let mut physical_file = self.physical_file.borrow_mut();
        physical_file.seek(SeekFrom::Start(16)).unwrap();
        physical_file
            .write_all(&file_table_page.to_le_bytes())
            .unwrap();
        physical_file
            .write_all(&file_table_size.to_le_bytes())
            .unwrap();
    }

    fn delete_recursive(&self, prev_index: u64, index: u64, is_parent_dir: bool) {
        let entity = self.get_file_struct_by_index(index);

        if entity.is_directory {
            let mut previous_pointer = index;
            let mut current_pointer = entity.contents;
            let previous_pointer_copy = previous_pointer;

            while current_pointer != 0 {
                let child_entity = self.get_file_struct_by_index(current_pointer);

                self.delete_recursive(
                    previous_pointer,
                    current_pointer,
                    previous_pointer == previous_pointer_copy,
                );

                previous_pointer = current_pointer;
                current_pointer = child_entity.next;
            }
            self.delete_single_dir(prev_index, index, is_parent_dir);
        } else {
            self.delete_single_file(prev_index, index, is_parent_dir);
        }
    }

    fn delete_single_dir(&self, prev_index: u64, index: u64, is_parent_dir: bool) {
        let mut dir = self.get_file_struct_by_index(index);
        dir.is_active = false;
        //if dir.is_root { println!("found root");}
        let mut prev = self.get_file_struct_by_index(prev_index);
        if is_parent_dir {
            prev.contents = dir.next;
        } else {
            prev.next = dir.next;
        }

        let modifications = vec![
            Modification::FileTable(index, dir),
            Modification::FileTable(prev_index, prev),
        ];
        self.schedule_commit(modifications);
        self.resolve_commit();
    }

    fn delete_single_file(&self, prev_index: u64, index: u64, is_parent_dir: bool) {
        let mut file: FileStruct = self.get_file_struct_by_index(index);
        file.is_active = false;
        let mut prev = self.get_file_struct_by_index(prev_index);
        if is_parent_dir {
            prev.contents = file.next;
        } else {
            prev.next = file.next;
        }

        let modifications = vec![
            Modification::AllcationTable(
                file.contents..(file.contents + ceil_div(file.size, self.page_size)),
                false,
            ),
            Modification::FileTable(index, file),
            Modification::FileTable(prev_index, prev),
        ];
        self.schedule_commit(modifications);
        self.resolve_commit();
    }
}

#[derive(Debug)]
enum Modification {
    SystemHeader(u64, u64),
    FileTable(u64, FileStruct),
    AllcationTable(std::ops::Range<u64>, bool),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    IO(std::io::ErrorKind),
    FileNameTooBig,
    FileStructSizeMismatch,
    IncompleteRead,
    IncompleteWrite,
    DirectoryNotFound,
    FileNotFound,
    NameAlreadyInUse,
    PageNumberTooBig,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value.kind())
    }
}

const FILE_NAME_SIZE: usize = 128;
const FILE_STRUCT_SIZE: usize = 1 + 5 * size_of::<u64>() + FILE_NAME_SIZE;
type FileBytes = [u8; FILE_STRUCT_SIZE];

#[derive(Debug, Clone, PartialEq)]
struct FileStruct {
    is_root: bool,
    is_active: bool,
    is_directory: bool,
    contents: u64,
    next: u64,
    size: u64, //number of bytes of actual file (divide by page_size to get number of pages)
    creation_time: u64,
    last_write_time: u64,
    name: String, //128 bytes
}

impl TryInto<FileBytes> for FileStruct {
    type Error = Error;

    fn try_into(self) -> Result<FileBytes, Self::Error> {
        if self.name.len() > FILE_NAME_SIZE {
            return Err(Error::FileNameTooBig);
        }

        let bytes = [0; FILE_STRUCT_SIZE];
        let mut output = Cursor::new(bytes);

        let mut flags = 0;
        flags |= self.is_root as u8;
        flags |= (self.is_active as u8) << 1;
        flags |= (self.is_directory as u8) << 2;

        let mut padded_name = [0u8; FILE_NAME_SIZE];

        for (i, &byte) in self.name.as_bytes().iter().enumerate() {
            padded_name[i] = byte;
        }

        output.write_all(&[flags]).unwrap();
        output.write_all(&self.contents.to_le_bytes()).unwrap();
        output.write_all(&self.next.to_le_bytes()).unwrap();
        output.write_all(&self.size.to_le_bytes()).unwrap();
        output.write_all(&self.creation_time.to_le_bytes()).unwrap();
        output
            .write_all(&self.last_write_time.to_le_bytes())
            .unwrap();
        output.write_all(&padded_name).unwrap();

        Ok(output.get_ref().to_owned())
    }
}

impl From<[u8; FILE_STRUCT_SIZE]> for FileStruct {
    fn from(value: [u8; FILE_STRUCT_SIZE]) -> Self {
        let mut input = Cursor::new(value);

        let mut flags = [0; 1];
        input.read_exact(&mut flags).unwrap();
        let mut contents = [0; size_of::<u64>()];
        input.read_exact(&mut contents).unwrap();
        let mut next = [0; size_of::<u64>()];
        input.read_exact(&mut next).unwrap();
        let mut size = [0; size_of::<u64>()];
        input.read_exact(&mut size).unwrap();
        let mut creation_time = [0; size_of::<u64>()];
        input.read_exact(&mut creation_time).unwrap();
        let mut last_write_time = [0; size_of::<u64>()];
        input.read_exact(&mut last_write_time).unwrap();
        let mut name = [0; FILE_NAME_SIZE];
        input.read_exact(&mut name).unwrap();

        let mut trimmed_name = name.to_vec();
        trimmed_name.retain(|&x| x != 0);

        FileStruct {
            is_root: (flags[0] & 1u8) != 0,
            is_active: (flags[0] & (1u8 << 1)) != 0,
            is_directory: (flags[0] & (1u8 << 2)) != 0,
            name: String::from_utf8(trimmed_name).unwrap(),
            contents: u64::from_le_bytes(contents),
            next: u64::from_le_bytes(next),
            size: u64::from_le_bytes(size),
            creation_time: u64::from_le_bytes(creation_time),
            last_write_time: u64::from_le_bytes(last_write_time),
        }
    }
}

#[derive(Debug)]
pub struct File {
    path: String,
    file_index: u64,
    cursor: u64,
    internal: Weak<RefCell<VfsInternal>>,
}

#[derive(Debug)]
pub struct Metadata {
    pub size: u64,
    pub last_write_time: u64,
    pub creation_time: u64,
}

impl File {
    fn get_metadata(&self) -> FileStruct {
        let upgrade = self.internal.upgrade().unwrap();
        let internal = upgrade.borrow_mut();
        internal.get_file_struct_by_index(self.file_index)
    }

    pub fn metadata(&self) -> Result<Metadata, Error> {
        {
            let upgrade = self.internal.upgrade();
            if upgrade.is_none() {
                return Err(Error::FileNotFound);
            }
            let upgrade = upgrade.unwrap();
            let internal = upgrade.borrow_mut();
            if internal.get_file_struct_by_path(&self.path).is_err() {
                return Err(Error::FileNotFound);
            }
        }

        let all_metadata = self.get_metadata();
        Ok(Metadata {
            size: all_metadata.size,
            last_write_time: all_metadata.last_write_time,
            creation_time: all_metadata.creation_time,
        })
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        {
            let upgrade = self.internal.upgrade();
            if upgrade.is_none() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
            let upgrade = upgrade.unwrap();
            let internal = upgrade.borrow_mut();
            if internal.get_file_struct_by_path(&self.path).is_err() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
        }

        let mut metadata = self.get_metadata();
        let upgrade = self.internal.upgrade().unwrap();
        let internal = upgrade.borrow_mut();
        if internal.get_file_struct_by_path(&self.path).is_err() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        }

        let number_of_pages_needed = ceil_div(self.cursor + buf.len() as u64, internal.page_size);
        let number_of_current_pages = ceil_div(metadata.size, internal.page_size);
        let overflow_condition = number_of_current_pages < number_of_pages_needed;

        // println!(
        //     "{} {} {} {} {}",
        //     overflow_condition,
        //     metadata.size,
        //     internal.page_size,
        //     number_of_current_pages,
        //     number_of_pages_needed
        // );

        if overflow_condition {
            let temp = internal.find_first_fitting_page_range(
                number_of_pages_needed,
                metadata.contents..(metadata.contents + number_of_current_pages),
            );
            if temp.is_none() {
                return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
            }
            let contents_location = temp.unwrap();

            //critical
            let mut modifications = Vec::new();

            modifications.push(Modification::AllcationTable(
                metadata.contents..(metadata.contents + number_of_current_pages),
                false,
            ));
            modifications.push(Modification::AllcationTable(
                contents_location..(contents_location + number_of_pages_needed),
                true,
            ));

            // internal
            //     .allocate_page_range(
            //         metadata.contents..(metadata.contents + number_of_current_pages),
            //         false,
            //     )
            //     .unwrap();
            // internal
            //     .allocate_page_range(
            //         contents_location..(contents_location + number_of_pages_needed),
            //         true,
            //     )
            //     .unwrap();

            let mut last_page = internal.get_number_of_pages();
            if metadata.contents <= last_page
                && last_page < metadata.contents + number_of_current_pages
            {
                last_page = metadata.contents - 1;
            }
            if last_page < contents_location + number_of_pages_needed - 1 {
                last_page = contents_location + number_of_pages_needed - 1;
            }
            let vfs_page_total = last_page + 1;
            // println!("{}", vfs_page_total);

            //println!("{} {} {} {}", metadata.contents, number_of_current_pages, contents_location, number_of_pages_needed);

            {
                let mut physical_file = internal.physical_file.borrow_mut();
                physical_file.set_len(vfs_page_total * internal.page_size)?;

                if metadata.size > 0 {
                    let mut contents_buffer: Vec<u8> = Vec::new();
                    contents_buffer.resize(metadata.size as usize, 0);
                    physical_file
                        .seek(SeekFrom::Start(metadata.contents * internal.page_size))
                        .unwrap();
                    physical_file.read_exact(&mut contents_buffer).unwrap();

                    physical_file
                        .seek(SeekFrom::Start(contents_location * internal.page_size))
                        .unwrap();
                    physical_file.write_all(&contents_buffer).unwrap();
                }
            }

            //critical

            metadata.contents = contents_location;

            modifications.push(Modification::FileTable(self.file_index, metadata.clone()));
            //internal.update_file_by_index(self.file_index, metadata.clone());

            internal.schedule_commit(modifications);
            internal.resolve_commit();
        }

        {
            let mut physical_file = internal.physical_file.borrow_mut();
            physical_file
                .seek(SeekFrom::Start(
                    metadata.contents * internal.page_size + self.cursor,
                ))
                .unwrap();
            physical_file.write_all(buf).unwrap();
        }

        //critical
        metadata.size = metadata.size.max(self.cursor + buf.len() as u64);
        metadata.last_write_time = VfsInternal::get_system_time();
        //internal.update_file_by_index(self.file_index, metadata.clone());

        let modifications = vec![Modification::FileTable(self.file_index, metadata)];
        internal.schedule_commit(modifications);
        internal.resolve_commit();

        self.cursor += buf.len() as u64;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let upgrade = self.internal.upgrade().unwrap();
        let internal = upgrade.borrow_mut();
        let mut physical_file = internal.physical_file.borrow_mut();
        physical_file.flush()
    }
}

//it should be able to read/write partially

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        {
            let upgrade = self.internal.upgrade();
            if upgrade.is_none() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
            let upgrade = upgrade.unwrap();
            let internal = upgrade.borrow_mut();
            if internal.get_file_struct_by_path(&self.path).is_err() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
        }
        let metadata = self.get_metadata();
        let readable_length = buf.len().min((metadata.size - self.cursor) as usize);

        if readable_length == 0 {
            return Ok(0);
        }

        let upgrade = self.internal.upgrade().unwrap();
        let internal = upgrade.borrow_mut();
        let mut physical_file = internal.physical_file.borrow_mut();

        physical_file
            .seek(SeekFrom::Start(
                metadata.contents * internal.page_size + self.cursor,
            ))
            .unwrap();
        let mut bytes = Vec::new();
        bytes.resize(readable_length, 0);
        physical_file.read_exact(&mut bytes).unwrap();

        for (i, &byte) in bytes.iter().enumerate() {
            buf[i] = byte;
        }

        self.cursor += readable_length as u64;

        Ok(readable_length)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        {
            let upgrade = self.internal.upgrade();
            if upgrade.is_none() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
            let upgrade = upgrade.unwrap();
            let internal = upgrade.borrow_mut();
            if internal.get_file_struct_by_path(&self.path).is_err() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
        }
        let metadata = self.get_metadata();
        match pos {
            SeekFrom::Start(x) if x < metadata.size => self.cursor = x,
            SeekFrom::Current(x)
                if 0 <= self.cursor as i128 + x as i128
                    && (self.cursor as i128 + x as i128) < metadata.size as i128 =>
            {
                self.cursor = (self.cursor as i128 + x as i128) as u64
            }
            SeekFrom::End(x) => self.cursor = (metadata.size as i128 + x as i128) as u64,
            _ => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
        }
        Ok(self.cursor)
    }
}

fn ceil_div(a: u64, b: u64) -> u64 {
    if a % b == 0 {
        a / b
    } else {
        a / b + 1
    }
}

#[derive(Debug)]
pub struct DirIterator {
    path: String,
    entry_names: Vec<String>,
    cursor: usize,
    internal: Weak<RefCell<VfsInternal>>,
}

#[derive(Debug)]
pub struct DirEntry {
    path: String,
}

impl DirEntry {
    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}

impl Iterator for DirIterator {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        {
            let upgrade = self.internal.upgrade();
            upgrade.as_ref()?;
            let upgrade = upgrade.unwrap();
            let internal = upgrade.borrow_mut();
            if internal.get_file_struct_by_path(&self.path).is_err() {
                return None;
            }
        }
        if self.cursor >= self.entry_names.len() {
            return None;
        }
        let path = self.path.clone() + "/" + &self.entry_names[self.cursor];
        self.cursor += 1;

        let upgrade = self.internal.upgrade().unwrap();
        let internal = upgrade.borrow();

        let entry = internal.get_file_struct_by_path(&path);

        if entry.is_err() {
            return Some(Err(Error::FileNotFound));
        }

        Some(Ok(DirEntry { path }))
    }
}
