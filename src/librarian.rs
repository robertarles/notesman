use std::io::{Write};
use std::fs;
use std::path::Path;
use chrono::{Datelike, Timelike, Local};

pub fn publish(filename: &String, lines: Vec<String>){
    let mut file = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&filename).unwrap();
    for line in lines {
        writeln!(file, "{}", line).expect(&format!("Unable to overwrite {}", filename));
    }
}

// backup, [create] and update a journal/archive file
// adds a hugo compat 'frontmatter' +++ header with a timestamp of the update time.
// each line being archived or journaled is prepended with a timestamp
pub fn archive(original_file: &String, backup_file: &String, new_lines: &Vec<String>, name: &str) {
    
    // create a current timestamp
    let now = Local::now();
    let (is_pm, hour) = now.hour12();
    let (_, year) = now.year_ce();
    let datestamp = format!(
        "{}-{:02}-{:02}", //, {:02}:{:02}{}UTC", // hugo header date format does not handle time
        year,
        now.month(),
        now.day()
    );
    let timestamp = format!("{:02}:{:02}{}", 
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
        let header_start_strings = vec!["+++", "Title =", "Date =", "Tags =", "Category ="];

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
    let mut file = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&original_file).unwrap();
    for line in appended_lines {
        writeln!(file, "{}", line).expect(&format!("Unable to overwrite {}", &original_file).to_string());
    }
}
