
fn main() {
    pretty_env_logger::init();

    let msg = "base binary not implemented";
    log::trace!("{}", msg);
    log::debug!("{}", msg);
    log::info!("{}", msg);
    log::warn!("{}", msg);
    log::error!("{}", msg);
}