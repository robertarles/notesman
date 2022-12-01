// section parsing info
pub struct NotesMetadata {
    pub journal_needle: String, // the string used to signify "journal this line"
    // special char sequences to identify lines
    pub journal_line_needle: String, // needles: what we search for to signify line properties
    pub archive_line_needle: String,
    pub list_line_needle: String,
    // markdown section identification (e.g. "## section name")
    pub todo_section_title_prefix: String,
    pub active_todo_section_title: String,
    pub done_todo_section_title: String,
    pub backlog_todo_section_title: String,
    // hugo front matter section identification (hugo site gen expects certain metadata here)
    pub front_matter_section_boundry: String,
    pub front_matter_date_key: String,
}
