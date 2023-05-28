enum Number {
	Time,
	Date,
	Size,
	Normal,
}

#[derive(Debug)]
pub enum Token {
	PerlRegex(String),
	PosixRegex(String),
	String(String),
	Time(Time),
	// Date(date),
}
