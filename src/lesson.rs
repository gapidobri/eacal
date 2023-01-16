use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Lesson {
    pub id: Option<String>,
    pub subject: String,
    pub classroom: String,
    pub teacher: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl Clone for Lesson {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            subject: self.subject.clone(),
            classroom: self.classroom.clone(),
            teacher: self.teacher.clone(),
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl PartialEq for Lesson {
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject
            && self.start == other.start
            && self.classroom == other.classroom
    }
}
