#[macro_use]
extern crate lazy_static;

use std::env;
use std::error::Error;
use std::time::Duration;

use sodiumoxide::crypto::auth::hmacsha256;
use tokio::runtime::Runtime;

use influx_receiver::error::{HMACKeyError, SodiumOxideError};

lazy_static! {
	static ref SKEY_NAME: &'static str = "INFLUX_SKEY";
	static ref DB_NAME: &'static str = "INFLUX_DB";
	static ref DB_PORT_NAME: &'static str = "INFLUX_PORT";
	static ref DB_ADDR_NAME: &'static str = "INFLUX_ADDR";
	static ref MAX_EXPIRATION_NAME: &'static str = "INFLUX_MAX_EXPIRATION";
	static ref MAX_EXPIRATION_DEFAULT: Duration = Duration::new(60, 0);
	static ref MINIMUM_WAIT_NAME: &'static str = "INFLUX_WAIT";
	static ref MINIMUM_WAIT_DEFAULT: f64 = 2.5f64;
}

fn main() -> Result<(), Box<dyn Error>> {
	let key = base64::decode(env::var(*SKEY_NAME)?)?;
	let db = env::var(*DB_NAME)?;
	let port = env::var(*DB_PORT_NAME)?;
	let addr = env::var(*DB_ADDR_NAME)?;
	let minimum_wait = env::var(*MINIMUM_WAIT_NAME).map_or(Ok(*MINIMUM_WAIT_DEFAULT), |val| val.parse::<_>())?;
	let max_expiration: Result<_, Box<dyn Error>> = env::var(*MAX_EXPIRATION_NAME).map_or(Ok(*MAX_EXPIRATION_DEFAULT), |val| Ok(Duration::new(val.parse::<_>()?, 0)));
	let max_expiration = max_expiration?;

	let key = hmacsha256::Key::from_slice(&key).ok_or(HMACKeyError::KeyLengthMismatch)?;

	let influx_client = influxdb::Client::new(format!("http://{}:{}", addr, port), db).with_auth("admin", "");
	let influx_runtime = Runtime::new()?;

	sodiumoxide::init().map_err(|_| SodiumOxideError::InitError)?;

	Err(influx_receiver::start(key, max_expiration, influx_client, influx_runtime, minimum_wait).into())
}
