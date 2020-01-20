use std::fs;
use std::io::{Write};
use std::fs::OpenOptions;
use std::path::Path;

extern crate clap;
use clap::{Arg, App};

extern crate chrono;
use chrono::{Datelike, Timelike, Utc};

//fn main()-> Result<(), Box<dyn std::error::Error>> {
fn main() {

    // handle the CLI args
    let cli_args = App::new("args")
        .arg(
            Arg::with_name("current_todo_file")
                .help("The todo file to be processed.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let passed_todo_filename = cli_args.value_of("current_todo_file").unwrap();
    
    // parse the passed filename (and path)
    let current_todo_filename: String = Path::new(&passed_todo_filename).file_name().unwrap().to_os_string().to_str().unwrap().to_string();
    let working_directory: String = Path::new(&passed_todo_filename).parent().unwrap().as_os_str().to_str().unwrap().to_string();

    // special strings / chars
    let journal_needle: &str = " . ";
    let journal_line_needle = format!("]{}", journal_needle);
    let archive_line_needle = "- [x] ";
    let list_line_needle = "- "; // this must be a starts_with on a trimmed line to catch indented list items

    // todo section tracking
    let mut current_section = "";
    // let header_tag = "#";
    let in_progress_section_title = "## TODO";
    let done_section_title = "## DONE";
    let archive_section_title = "## ARCHIVE";
    // let backlog_section_title = "## BACKLOG"; no special treatment for this section, title not used here

    // output file text
    let mut journal_lines: Vec<String> = vec![];
    let mut todo_lines: Vec<String> = vec![];
    let mut archive_lines: Vec<String> = vec![];

    // default OUT FILE names
    let journal_filename = current_todo_filename.replace(".md", "-JOURNAL.md");
    let archive_filename = current_todo_filename.replace(".md", "-ARCHIVE.md");

    // create a timestamp for the archived and jouranled items
    let now = Utc::now();
    let (is_pm, hour) = now.hour12();
    let (_, year) = now.year_ce();
    let timestamp = format!(
        "[{}{:02}{:02}T{:02}:{:02}{}UTC]",
        year,
        now.month(),
        now.day(),
        hour,
        now.minute(),
        if is_pm { "PM" } else { "AM" }
    );
    let journal_stamp = format!(" {} ",&timestamp);
    let archive_stamp = format!("{} ",&timestamp);

    let current_todo_infile = format!("{}/{}", &working_directory, &current_todo_filename);
    let todo_backup_filename = format!("{}/.{}.bak", &working_directory, &current_todo_filename);

    println!("Reading [{}]", &current_todo_infile);
    let current_todo_lines_str = fs::read_to_string(&current_todo_infile).expect("Unable to read current todo file");
    let current_todo_lines = current_todo_lines_str.split("\n") ;
    
    for line in current_todo_lines {

        // when finding a header/section, clear the section. All code must assume we're not in a special section at this point.
        if line.starts_with("#") {
            current_section = "";
        }
        // we're in the in-progress section
        if line.starts_with(in_progress_section_title){
            current_section = in_progress_section_title;
        }
        // sections archive && done have the same use case
        if line.starts_with(archive_section_title) {
            current_section = archive_section_title;
        }
        if line.starts_with(done_section_title) {
            current_section = done_section_title
        }

        // if in progress, journal items with journal mark and archive items that are complete
        if current_section == in_progress_section_title {
            
            // journal lines with the journal mark
            if line.contains(&journal_line_needle) {
                journal_lines.push(line.replace(journal_needle, &journal_stamp));
            }

            // if closed, move to archive, else keep in todo
            if line.contains(&archive_line_needle) {
                archive_lines.push(line.replace(&archive_line_needle, &archive_stamp).to_string());
            }else{
                todo_lines.push(line.replace(journal_needle, " ")); // needle is surrounded by spaces, leave one there
            }
        
        // archive all list items in an archive-type section
        } else if current_section == archive_section_title || current_section == done_section_title {
            if line.contains(&archive_line_needle) {
                archive_lines.push(line.replace(&archive_line_needle, &archive_stamp).to_string());
            } else if line.contains(list_line_needle) {
                archive_lines.push(line.replace(&list_line_needle, &archive_stamp).to_string());
            } else {
                todo_lines.push(line.to_string());
            }
            
        } else {
            // all "not special" lines stay in the todo file
            todo_lines.push(line.to_string());
        }
    }

    // *****
    // MAINTAIN JOURNAL FILES
    // *****

    // if exists, read the current journal, append it to the new journal lines
    let journal_original_file = format!("{}/{}", &working_directory, &journal_filename);
    let journal_backup_file = format!("{}/.{}.bak", &working_directory, &journal_filename);
    if Path::new(&journal_original_file).exists() {
        // create back up the current journal
        // OpenOptions::new().create(true).write(true).truncate(true).open(&journal_backup_file).expect("failed to create backup journal file");
        // println!("DEBUG \n\t{}\n\t{}", &journal_original_file, &journal_backup_file);
        fs::copy(&journal_original_file, &journal_backup_file).expect("failed making a backup of the journal file");
        // append the current journal to the new journal lines (latest placed at top of journal)
        let current_journal_lines = fs::read_to_string(&journal_original_file).expect("Unable to read current version of journal file");   
        for line in current_journal_lines.split("\n") {
            journal_lines.push(line.to_string());
        }
    }

    // overwrite the current journal with the new+old lines
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(format!("{}/{}", &working_directory, &journal_filename)).unwrap();
    for line in &journal_lines{
        writeln!(file, "{}", line).expect("Unable to overwrite the journal data");
    }
    
    // *****
    // MAINTAIN ARCHIVE FILES
    // *****

    // read the current archive, append it to the new archive lines
    let archive_current_file = format!("{}/{}", &working_directory, &archive_filename);
    let archive_backup_file = format!("{}/.{}.bak", &working_directory, &archive_filename);
    if Path::new(&archive_current_file).exists() {
        // back up the current archive
        //OpenOptions::new().create(true).write(true).truncate(true).open(&archive_backup_file).expect("failed to create backup archive file");
        fs::copy(&archive_current_file, &archive_backup_file).expect("failed making a backup of the archive file");
        // append the current archive to the new archive lines (latest placed at top of archives)
        let current_archive_lines = fs::read_to_string(&archive_current_file).expect("Unable to read current version of archive file");   
        for line in current_archive_lines.split("\n") {
            archive_lines.push(line.to_string());
        }
    }

    // overwrite the current archive with the new+old lines
    overwrite_file(&archive_current_file, archive_lines);

    // *****
    // UPDATE THE TODO FILE
    // *****

    // backup the current todos
    fs::copy(&current_todo_infile, &todo_backup_filename).expect("failed to write the updates to the todo file");
    
    // re-write the todo
    overwrite_file(&current_todo_infile, todo_lines);

}

fn overwrite_file(filename: &String, lines: Vec<String>){
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&filename).unwrap();
    for line in lines {
        writeln!(file, "{}", line).expect(&format!("Unable to overwrite {}", filename));
    }
}