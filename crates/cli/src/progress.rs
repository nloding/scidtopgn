pub fn create_progress_bar(total: usize) -> indicatif::ProgressBar {
    indicatif::ProgressBar::new(total as u64)
}
