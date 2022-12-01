use std::fs;
use std::path::Path;

mod cli;
mod librarian;
mod notes_utils;
use notes_utils::NotesMetadata;

extern crate chrono;
use chrono::{Datelike, Local, Timelike};

fn main() {
    // parse the cli args
    let cli_args = cli::arg_handler();

    // get the passed in filename parameter
    let passed_todo_filename = cli_args.value_of("todo_file").unwrap();
    // make sure a markdown file was specified (TODO: add handling of arbitrary extensions?)
    if !passed_todo_filename.ends_with(".md") {
        std::process::exit(1);
    }
    // set the extenstion expected for use in creating journal, archive and backup filestodo_section_title
    let file_extension = ".md";

    // parse the passed filename and path
    let original_todo_filename: String = Path::new(&passed_todo_filename)
        .file_name()
        .unwrap()
        .to_os_string()
        .to_str()
        .unwrap()
        .to_string();
    // get the path to the specified todo file
    let working_directory: String = Path::new(&passed_todo_filename)
        .parent()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    // vars to gather output file text
    let mut journal_lines: Vec<String> = vec![];
    let mut todo_lines: Vec<String> = vec![];
    let mut archive_lines: Vec<String> = vec![];

    // default output file names - append -JOURNAL|-ARCHIVE to the base names (maintaining extension)
    let journal_filename =
        original_todo_filename.replace(file_extension, &format!(".journal{}", &file_extension));
    let archive_filename =
        original_todo_filename.replace(file_extension, &format!(".archive{}", &file_extension));

    let notes_meta = NotesMetadata {
        journal_needle: String::from(" . "), // "touched" mark, journal worked on / "touched" items
        journal_line_needle: String::from("] . "), // "touched, a journal-ready line will have a checkbox and then the journal indicator
        archive_line_needle: String::from("- [x] "), // a checked checkbox, done
        list_line_needle: String::from("- "),      // markdown syntax, this specifies a list item
        todo_section_title_prefix: String::from("## "), 
        active_todo_section_title: String::from("## ACTIVE"),
        backlog_todo_section_title: String::from("## BACKLOG"),
        done_todo_section_title: String::from("## DONE"), // every line placed in a DONE or ARCHIVE section should be archived
        front_matter_section_boundry: String::from("---"), // this is the header section (front matter) boundry marker for HUGO static site builder
        front_matter_date_key: String::from("date:"), // we'll update date in the header each time we process a todo file
    };

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
    let datestring = format!("{}-{:02}-{:02}", year, now.month(), now.day(),);
    let journal_line_stamp = format!(" {} ", &timestamp);
    let archive_line_prefix = format!("- {} ", &timestamp); // make each line a list item (markdown otherwise globs all lines into a paragraph)

    // the todo file to process
    let todo_source_filename = format!("{}/{}", &working_directory, &original_todo_filename);
    // the backup file to write (working with a safety net!)
    let todo_backup_filename = format!("{}/.{}.bak", &working_directory, &original_todo_filename);

    println!("Reading [{}]", &todo_source_filename);
    let todo_lines_str =
        fs::read_to_string(&todo_source_filename).expect("Unable to read specified todo file");

    // loop over each line in the ToDo file
    let mut current_section = String::from(""); // start with no section found
    for line in todo_lines_str.split("\n") {
        // when finding a header/section, clear the section. All code must assume we're not in a special section at this point.
        if line.starts_with(&notes_meta.todo_section_title_prefix) {
            current_section = "".to_string();
        }
        // we're in the in-progress section
        if line.starts_with(&notes_meta.active_todo_section_title) {
            current_section = notes_meta.active_todo_section_title.to_string();
        }
        // we're in the backlog section
        if line.starts_with(&notes_meta.backlog_todo_section_title) {
            current_section = notes_meta.backlog_todo_section_title.to_string();
        }
        // we're in the done section
        if line.starts_with(&notes_meta.done_todo_section_title) {
            current_section = notes_meta.done_todo_section_title.to_string();
        }
        // front matter starts and ends with the same 'front_matter_section_boundry',
        // 'toggle off' the section if we see it again
        if line.starts_with(&notes_meta.front_matter_section_boundry)
            && (current_section != notes_meta.front_matter_section_boundry)
        {
            current_section = notes_meta.front_matter_section_boundry.to_string();
        }

        /*
         * begin processing lines
         */
        if current_section == notes_meta.front_matter_section_boundry {
            // update the date line in the frontmatter (header) section
            if line
                .to_lowercase()
                .starts_with(&notes_meta.front_matter_date_key.to_lowercase())
            {
                todo_lines.push(format!(
                    "{} \"{}\"",
                    &notes_meta.front_matter_date_key, datestring
                ));
            } else {
                todo_lines.push(line.to_string());
            }

        // journal items with journal mark (those marked touched), plus archive ALL items that are marked complete
        } else if current_section == notes_meta.active_todo_section_title {
            // journal lines with the journal mark
            if line.contains(&notes_meta.journal_line_needle) {
                journal_lines.push(line.replace(&notes_meta.journal_needle, &journal_line_stamp));
            }

            // if closed, move to archive, else keep in todo
            if line.contains(&notes_meta.archive_line_needle) {
                archive_lines.push(
                    line.replace(&notes_meta.archive_line_needle, &archive_line_prefix)
                        .to_string(),
                );
            } else {
                todo_lines.push(line.replace(&notes_meta.journal_needle, " ")); // needle is surrounded by spaces, leave one space there
            }

        // archive all list items in an archive-type section
        } else if current_section == notes_meta.backlog_todo_section_title
            || current_section == notes_meta.done_todo_section_title
        {
            if line.contains(&notes_meta.archive_line_needle) {
                // archive completed items even in if the backlog section
                archive_lines.push(
                    line.replace(&notes_meta.archive_line_needle, &archive_line_prefix)
                        .to_string(),
                );
            //} else if line.contains(&notes_meta.list_line_needle) {
            // TODO: why was this archiving list items in the backlog?
            //archive_lines.push(line.replace(&notes_meta.list_line_needle, &archive_line_prefix).to_string());
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
    librarian::archive(
        &journal_original_file,
        &journal_backup_file,
        &journal_lines,
        &"journal"[..],
    );

    // *****
    // MAINTAIN ARCHIVE FILES
    // *****

    // read the current archive, append it to the new archive lines
    let archive_original_file = format!("{}/{}", &working_directory, &archive_filename);
    let archive_backup_file = format!("{}/.{}.bak", &working_directory, &archive_filename);
    librarian::archive(
        &archive_original_file,
        &archive_backup_file,
        &archive_lines,
        &"archive"[..],
    );

    // *****
    // UPDATE THE todo FILE
    // *****

    // backup the current todos
    fs::copy(&todo_source_filename, &todo_backup_filename)
        .expect("failed to write the updates to the todo file");

    // update the todo
    librarian::publish(&todo_source_filename, todo_lines);
}
