use std::fs;
use std::path::Path;

mod cli;
mod librarian;

extern crate chrono;
use chrono::{Datelike, Timelike, Local};

fn main() {

    // setup arg handling, get the todo filename
    let cli_args = cli::arg_handler();

    let passed_todo_filename = cli_args.value_of("todo_file").unwrap();
    // make sure a markdown file was specified (TODO: add handling of arbitrary extensions?)
    if ! passed_todo_filename.ends_with(".md") {
        std::process::exit(1);
    }
    // set the extenstion expected for use in creating journal, archive and backup files
    let file_extension = ".md";
    
    // parse the passed filename and path
    let original_todo_filename: String = Path::new(&passed_todo_filename).file_name().unwrap().to_os_string().to_str().unwrap().to_string();
    // get the path to the specified todo file
    let working_directory: String = Path::new(&passed_todo_filename).parent().unwrap().as_os_str().to_str().unwrap().to_string();

    // special char sequences to identify lines
    let journal_needle: &str = " . ";  // the "I workied on this, please journal it" indicator
    let journal_line_needle = format!("]{}", journal_needle); // a journal-ready line will have a checkbox and then the journal indicator
    let archive_line_needle = "- [x] "; // a checked checkbox
    let list_line_needle = "- "; // markdown syntax, this specifies a list item

    // todo section tracking
    let mut current_section = ""; // start with no section found
    let todo_section_title = "## TODO";
    let done_section_title = "## DONE"; // every line placed in a DONE or ARCHIVE section should be archived
    let archive_section_title = "## ARCHIVE";
    let front_matter_section_boundry = "+++"; // this is the header section (front matter) boundry marker for HUGO static site builder
    let front_matter_date_key = "Date="; // we'll update date in the header each time we process a todo file

    // vars to gather output file text
    let mut journal_lines: Vec<String> = vec![];
    let mut todo_lines: Vec<String> = vec![];
    let mut archive_lines: Vec<String> = vec![];

    // default output file names for journaling and archiving data
    let journal_filename = original_todo_filename.replace(file_extension, &format!("-JOURNAL{}", &file_extension));
    let archive_filename = original_todo_filename.replace(file_extension, &format!("-ARCHIVE{}", &file_extension));

    // create a timestamp for the archived and journaled items/lines
    let now = Local::now();
    let (is_pm, hour) = now.hour12();
    let (_, year) = now.year_ce();
    let timestamp = format!(
        "[{}-{:02}-{:02}, {:02}:{:02}{}]",
        year,
        now.month(),
        now.day(),
        hour,
        now.minute(),
        if is_pm { "pm" } else { "am" }
    );
    let datestring = format!(
        "{}-{:02}-{:02}",
        year,
        now.month(),
        now.day(),
    );
    let archive_line_stamp = format!(" {} ",&timestamp);
    let archive_line_prefix = format!("- {} ",&timestamp); // make each line a list item (markdown otherwise globs all lines into a paragraph)

    let todo_infile = format!("{}/{}", &working_directory, &original_todo_filename);
    let todo_backup_filename = format!("{}/.{}.bak", &working_directory, &original_todo_filename);

    println!("Reading [{}]", &todo_infile);
    let todo_lines_str = fs::read_to_string(&todo_infile).expect("Unable to read specified todo file");

    for line in todo_lines_str.split("\n") {

        // when finding a header/section, clear the section. All code must assume we're not in a special section at this point.
        if line.starts_with("#") {
            current_section = "";
        }
        // we're in the in-progress section
        if line.starts_with(todo_section_title){
            current_section = todo_section_title;
        }
        // sections archive && done have the same use case
        if line.starts_with(archive_section_title) {
            current_section = archive_section_title;
        }
        if line.starts_with(done_section_title) {
            current_section = done_section_title
        }
        // front matter starts and ends with the same 'front_matter_section_boundry', so 'toggle off' the section if we see it again
        if line.starts_with(front_matter_section_boundry) && (current_section != front_matter_section_boundry) {
            current_section = front_matter_section_boundry
        }

        /*
        * begin processing lines
        */
        if current_section == front_matter_section_boundry {

            // update the date line in the frontmatter (header) section
            if line.starts_with(&front_matter_date_key) {
                todo_lines.push(format!("{} \"{}\"", front_matter_date_key, datestring));
            } else {
                todo_lines.push(line.to_string());
            }

        // journal items with journal mark (those marked touched), plus archive ALL items that are marked complete
        } else if current_section == todo_section_title {
            
            // journal lines with the journal mark
            if line.contains(&journal_line_needle) {
                journal_lines.push(line.replace(journal_needle, &archive_line_stamp));
            }

            // if closed, move to archive, else keep in todo
            if line.contains(&archive_line_needle) {
                archive_lines.push(line.replace(&archive_line_needle, &archive_line_prefix).to_string());
            }else{
                todo_lines.push(line.replace(journal_needle, " ")); // needle is surrounded by spaces, leave one space there
            }
        
        // archive all list items in an archive-type section
        } else if current_section == archive_section_title || current_section == done_section_title {
            if line.contains(&archive_line_needle) {
                archive_lines.push(line.replace(&archive_line_needle, &archive_line_prefix).to_string());
            } else if line.contains(list_line_needle) {
                archive_lines.push(line.replace(&list_line_needle, &archive_line_prefix).to_string());
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
    librarian::archive(&journal_original_file, &journal_backup_file, &journal_lines, &"journal"[..]);

    // *****
    // MAINTAIN ARCHIVE FILES
    // *****

    // read the current archive, append it to the new archive lines
    let archive_original_file = format!("{}/{}", &working_directory, &archive_filename);
    let archive_backup_file = format!("{}/.{}.bak", &working_directory, &archive_filename);
    librarian::archive(&archive_original_file, &archive_backup_file, &archive_lines, &"archive"[..]);

    // *****
    // UPDATE THE TODO FILE
    // *****

    // backup the current todos
    fs::copy(&todo_infile, &todo_backup_filename).expect("failed to write the updates to the todo file");

    // update the todo
    librarian::publish(&todo_infile, todo_lines);

}