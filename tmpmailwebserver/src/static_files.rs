use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static"]  // 你的静态文件目录
pub struct Asset;