mod ice;
mod rtc;
mod sdp;

pub use rtc::Rtc;

#[derive(Clone, Debug)]
pub enum Error {
    ConfigurationError,
    ConnectionError,
}
