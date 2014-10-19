use std::io::File;
use std::io::fs::walk_dir;
use std::io::fs::copy;
use std::io::fs::rename;
use std::io::fs::stat;
use std::io::fs::change_file_times;
use std::io::TempDir;
use std::io::fs::PathExtensions;
use std::io::Command;

#[test]
fn test_renames_via_single_process() {
    test_renames("");
}

#[test]
fn test_renames_via_external_process() {
    test_renames("--use-external-process");
}

#[test]
fn test_incorrect_option() {

    let mut process = Command::new("./target/mvsync")
        .arg("/source")
        .arg("/destination")
        .arg("--this-flag-does-not-exists")
        .spawn()
        .unwrap();

   assert!(process.wait().unwrap().success() == false);
}

fn test_renames<T: ToCStr>(argument: T) {

    // Create a temporary directory and write some
    // files into it
    let original_directory = TempDir::new("").unwrap();
    write_file(original_directory.path(), "file1".to_string(), "000".to_string());
    write_file(original_directory.path(), "file2".to_string(), "1111".to_string());
    write_file(original_directory.path(), "file3".to_string(), "22222".to_string());

    // Copy all those files into a new directory
    let new_directory = TempDir::new("").unwrap();
    copy_directory_contents(original_directory.path(), new_directory.path());

    // Rename some of the files of the original directory
    rename_keeping_modified_time(
        &original_directory.path().join("file1"),
        &original_directory.path().join("file7"));
    rename_keeping_modified_time(
        &original_directory.path().join("file2"),
        &original_directory.path().join("file11"));

    // Run mvsync
    let mut process = match Command::new("./target/mvsync")
        .arg(original_directory.path().as_str().unwrap())
        .arg(new_directory.path().as_str().unwrap()) 
        .arg(argument)
        .spawn()
        {
            Ok(process) => process,
            Err(e) => fail!("failed to execute process: {}", e),
        };

    // Set a timeout of 4 seconds
    process.set_timeout(Some(4_000));

    match process.wait()
    {
        Ok(status) => 
        {
            println!("{}", process.stdout.as_mut().unwrap().read_to_string().unwrap());
            println!("{}", process.stderr.as_mut().unwrap().read_to_string().unwrap());
        },
        Err(e) => 
        {
            process.signal_kill();
            println!("{}", process.stdout.as_mut().unwrap().read_to_string().unwrap());
            println!("{}", process.stderr.as_mut().unwrap().read_to_string().unwrap());
            fail!("failed to execute process: {}", e)
        }
    }

    // Get the output
    //let output = match process.wait_with_output()
    //{
    //    Ok(output) => output,
    //    Err(e) => {
    //        fail!("process timemout: {}", e);
    //    }
    //};

    //println!("status: {}", output.status);
    //println!("stdout: {}", String::from_utf8_lossy(output.output.as_slice()));
    //println!("stderr: {}", String::from_utf8_lossy(output.error.as_slice()));
    
    // Verify that the new directory contents have been renamed
    // accordingly
    assert!(new_directory.path().join("file1").exists() == false);    
    assert!(new_directory.path().join("file2").exists() == false);    
    assert!(new_directory.path().join("file7").exists() == true);    
    assert!(new_directory.path().join("file11").exists() == true);    
}

fn write_file(directory: &Path, file_name: String, contents: String) {
    
    let full_path = directory.join(file_name);
    let mut file = File::create(&full_path);
    file.write(contents.as_bytes()).unwrap();

}

fn copy_directory_contents(source: &Path, destination: &Path) {

   for item in walk_dir(source).unwrap() {
       let relative_path = item.path_relative_from(source).unwrap();
       let destination_path = destination.join(relative_path);
       copy(&item, &destination_path).unwrap();
       let metadata = stat(&item).unwrap();
       change_file_times(&item, metadata.accessed, metadata.modified).unwrap();
       change_file_times(&destination_path, metadata.accessed, metadata.modified).unwrap();
   }

}

fn rename_keeping_modified_time(source: &Path, destination: &Path) {

    let metadata = stat(source).unwrap();
    rename(source, destination).unwrap();
    change_file_times(destination, metadata.accessed, metadata.modified).unwrap();
}
