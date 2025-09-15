use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProtonApp {
    pub appname: String,
    pub appid: String,
    pub icon: String,
    pub lastplaytime: String,
    pub exe: String,
    pub startdir: String,
}
