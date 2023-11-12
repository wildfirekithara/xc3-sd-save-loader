macro_rules! dbg_println {
  ($($arg:tt)*) => {{
      crate::logging::write_log_line(&format!($($arg)*));
  }};
}

pub(crate) use dbg_println;

pub fn write_log_line(line: &str) {
  let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
  println!("{}", format!("[{}] [XC3-SD-Save-Loader] {}", ts, line));
}
