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
                .help("The source todo file to be processed.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let passed_todo_filename = cli_args.value_of("current_todo_file").unwrap();
    
    // parse the passed filename and path
    let current_todo_filename: String = Path::new(&passed_todo_filename).file_name().unwrap().to_os_string().to_str().unwrap().to_string();
    let working_directory: String = Path::new(&passed_todo_filename).parent().unwrap().as_os_str().to_str().unwrap().to_string();

    // special char sequences to identify lines
    let journal_needle: &str = " . ";
    let journal_line_needle = format!("]{}", journal_needle);
    let archive_line_needle = "- [x] ";
    let list_line_needle = "- "; // this should be a starts_with on a trimmed line to catch indented list items

    // todo section tracking
    let mut current_section = "";
    let in_progress_section_title = "## TODO";
    let done_section_title = "## DONE";
    let archive_section_title = "## ARCHIVE";
    // let header_tag = "#";
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
    let archive_stamp = format!("- {} ",&timestamp);

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

        // journal items with journal mark (those marked touched), plus archive ALL items that are marked complete
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
            // all "do-not-move-to-archive" lines stay in the todo file
            todo_lines.push(line.to_string());
        }
    }
    //println!("{:#?}", journal_lines);
     // *****
     // MAINTAIN JOURNAL FILES
     // *****

     // if exists, read the current journal, append it to the new journal lines
     let journal_original_file = format!("{}/{}", &working_directory, &journal_filename);
     let journal_backup_file = format!("{}/.{}.bak", &working_directory, &journal_filename);
     process_secondary(&journal_original_file, &journal_backup_file, &journal_lines, &"journal"[..]);
    
     // *****
     // MAINTAIN ARCHIVE FILES
     // *****

     // read the current archive, append it to the new archive lines
     let archive_original_file = format!("{}/{}", &working_directory, &archive_filename);
     let archive_backup_file = format!("{}/.{}.bak", &working_directory, &archive_filename);
     process_secondary(&archive_original_file, &archive_backup_file, &archive_lines, &"archive"[..]);

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

// backup, [create] and update a journal/archive file
// adds a hugo compat 'frontmatter' +++ header with a timestamp of the update time.
// each line being archived or journaled is prepended with a timestamp
fn process_secondary(original_file: &String, backup_file: &String, new_lines: &Vec<String>, name: &str) {
    
    // create a current timestamp
    let now = Utc::now();
    let (is_pm, hour) = now.hour12();
    let (_, year) = now.year_ce();
    let datestamp = format!(
        "{}-{:02}-{:02}", //, {:02}:{:02}{}UTC", // hugo header date format does not handle time
        year,
        now.month(),
        now.day()
    );
    let timestamp = format!("{}:{}{}", 
        hour,
        now.minute(),
        if is_pm { "PM" } else { "AM" }
    );

    // set a line to this value before the push, and it will not be pushed.
    let delete_line_mark = "+++DELETE_LINE+++";

    let mut appended_lines = vec![];
    // add a header and a blank line to make the markdown valid
    let hugo_header: String = String::from(format!("+++\nTitle = \"TODO {}, {} {}\"\nDate = \"{}\"\nTags = [\"TODO-{}\"]\n+++", &name, &datestamp, &timestamp, &datestamp, &name));
    appended_lines.push(hugo_header);

    //appended_lines.push(format!("# last updated {}\n", &datestamp));
    for line in new_lines {
            if line.starts_with("- ") {
                appended_lines.push(line.to_string());
            }else{
                appended_lines.push(format!("- {}", line.to_string()));
            }
    }

    if Path::new(&original_file).exists() {
        // create back up the current file
        fs::copy(&original_file, &backup_file).expect(&format!("failed making a backup of {}", &original_file).to_string());
        // append the current file to the new file lines (latest placed at top of file)
        let original_lines = fs::read_to_string(&original_file).expect(&format!("Unable to read contents of {}", &original_file).to_string());  

        let mut in_header_section = false;
        let header_start_strings = vec!["+++", "Title =", "Date =", "Tags ="];

        for mut line in original_lines.split("\n") {

            // when in the header, mark line for deletion, skip to next line
            if line.starts_with("+++"){
                // toggle true/false
                in_header_section = if in_header_section == false { true } else { false };
                // if this is a simple header marker, set it to be deleted (guessing that it may have a space or two following it)
                if line.len() < 5 {
                    line = delete_line_mark;
                }
            }

            // don't assume our header markup is reliable, also check that the line looks like a header
            if in_header_section {
                for header_start_string in &header_start_strings{
                    if line.starts_with(header_start_string){
                        // delete this line, we don't keep old headers
                        line = delete_line_mark;
                    }
                }
            }

            // if it's "deleted" or a (nearly)blank line
            if line != delete_line_mark && line.len() > 2 {
                // if it's already a list item just add it (md to html output is nicer this way)
                if line.starts_with("- ") {
                    appended_lines.push(line.to_string());
                }else{
                    appended_lines.push(format!("- {}", line.to_string()));
                }
            }
        }

    }

    // overwrite the current file with the new+old lines
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&original_file).unwrap();
    for line in appended_lines {
        writeln!(file, "{}", line).expect(&format!("Unable to overwrite {}", &original_file).to_string());
    }
}