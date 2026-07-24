use heapless::String;

const MAX_LINE_LENGTH: usize = 255;
const MAX_FILE_NAME: usize = 12;

pub trait SDPayload {
	// Returns file name & line respectively
	fn serialize() -> (String<MAX_FILE_NAME>, String<MAX_LINE_LENGTH>);
}
