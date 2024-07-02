const LOG_LEVEL: u8 = 3;

pub fn j_log(str: &str, log_level: u8) {
    if log_level <= LOG_LEVEL {
        print!("LOGGING (lv{}): {}\n", log_level, str);
    }
}
