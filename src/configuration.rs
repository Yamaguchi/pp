use crate::errors::Error;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub application: Application,
    pub grpc: gRPC,
    pub server: Server,
}

#[derive(Debug, Deserialize)]
pub struct Application {
    pub private_key: String,
}

#[derive(Debug, Deserialize)]
pub struct gRPC {
    pub bind: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub bind: String,
}

impl Configuration {
    pub fn new(path: String) -> Result<Self, Error> {
        Configuration::read(path)
    }

    fn read(path: String) -> Result<Configuration, Error> {
        let mut file_content = String::new();
        let mut fr = File::open(path)
            .map(|f| BufReader::new(f))
            .map_err(|e| Error::CannotRead(e))?;
        fr.read_to_string(&mut file_content)
            .map_err(|e| Error::CannotRead(e))?;
        let config = toml::from_str(&file_content).map_err(|e| Error::CannnotParseConfigFile(e))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let config = Configuration::read("./config.sample.toml".to_string());
        assert_eq!(
            config.unwrap().application.private_key,
            "649293486b0d3af1f90243021453dcb7dbbbd9fd3a54c373eaca02d230aa3154"
        );
    }
}
