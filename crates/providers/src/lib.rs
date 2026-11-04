pub mod innertube;
pub mod vk;
pub mod soundcloud;
pub mod lossless;
pub mod bandcamp;
pub mod audius;

pub use innertube::InnerTubeProvider;
pub use vk::VkSourceProvider;
pub use soundcloud::SoundCloudProvider;
pub use lossless::LosslessMirrorProvider;
pub use bandcamp::BandcampProvider;
pub use audius::AudiusProvider;
