use chrono::Datelike;

fn main() {
    // get terminal cols
    let cols = match termsize::get() {
        None => 0,
        Some(s) => s.cols,
    };

    let mut pb = pbr::ProgressBar::new(100);
    pb.format("[#--]");
    pb.show_speed = false;
    pb.show_percent = false;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.show_tick = false;
    pb.show_message = false;
    if cols != 0 {
        pb.set_width(Some(cols as usize))
    }

    let now = chrono::offset::Local::now();
    let days = now.clone().ordinal0() as u64;

    pb.set(days * 100 / 365);
}
