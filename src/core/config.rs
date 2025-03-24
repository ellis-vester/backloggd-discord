use std::fs;

pub fn get_docker_file_secret(path: &str) -> Result<String, anyhow::Error> {
    let secret = fs::read_to_string(path).expect("Unable to read secret at {path}");
    return Ok(secret);
}
