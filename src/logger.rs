pub fn init() {
    com_logger::builder()
        .filter(log::LevelFilter::Trace)
        .setup();
}
