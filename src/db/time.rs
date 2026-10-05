use chrono::Local;




pub fn now() -> String {
    let dt = Local::now(); // local timezone
    dt.format("%d %B, %Y, %I:%M %p").to_string()
}




