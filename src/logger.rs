pub fn initialize_logger() {
    com_logger::builder()
        .filter(log::LevelFilter::Trace)
        .setup();
}
