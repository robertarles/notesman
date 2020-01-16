use std::fs;

fn main() {
    let touched_char = "."
    let mut journal_lines: Vec<&str> = vec![];
    let data = fs::read_to_string("/Users/robert/Documents-DISNEY/NOTES/DISNEY/todos-DIS.md").expect("Unable to read file");
    let data_lines: Vec<&str> = data.split('\n').collect();
    for line in data_lines.iter() {
        let touched_line_needle = format!("] {}", touched_char);
        if line.contains(&touched_line_needle) {
            journal_lines.push(line);
            // TODO: modify line to remove "touched" character
            line = str::replace(line, &touched_line_needle, ""); 
        }
    }
    for line in journal_lines.iter() {
        println!("-> {}", line);
    }
    // TODO: write journal to outfile
    // TODO: on successful journaling, write modified infile
}
