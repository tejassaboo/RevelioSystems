#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::Cursor;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use influxdb::{Client, InfluxDbWriteable, Timestamp, WriteQuery};
use rocket::{Request, Response, response, State};
use rocket::error::LaunchError;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, ResponseBuilder};
use rocket_contrib::json::Json;
use serde::Deserialize;
use sodiumoxide::crypto::auth::hmacsha256;
use tokio::runtime::Runtime;

use error::AuthenticateError;

pub mod error;

impl<'r> Responder<'r> for AuthenticateError {
	fn respond_to(self, _: &Request) -> response::Result<'r> {
		build_response(Status::Unauthorized, &format!("{}", self)).ok()
	}
}

fn build_response<'r>(status: Status, err_msg: &str) -> ResponseBuilder<'r> {
	let error = json!({"message": err_msg,});

	let mut builder = Response::build();
	builder
		.status(status)
		.header(ContentType::JSON)
		.sized_body(Cursor::new(error.to_string() + "\n"));
	builder
}

#[derive(Deserialize)]
struct Assertion {
	message: String,
	sig: String,
}

#[derive(Deserialize)]
struct Message {
	nonce: u128,
	expires: Duration,
	payload: Metrics,
}

#[derive(Deserialize)]
struct Metrics {
	time: u128,
	duration: u64,
	gateway: bool,
	method: Method,
	uri: String,
	name: String,
	id: u128,
	tcpip: TCPIP,
}

#[derive(Deserialize)]
struct TCPIP {
	src: IpAddr,
	dst: IpAddr,
	sport: u16,
	dport: u16,
}

impl Metrics {
	fn into_query(self) -> WriteQuery {
		let time = Timestamp::Nanoseconds(self.time);
		let query = time.into_query(self.name)
			.add_field("duration", self.duration)
			.add_tag("gateway", self.gateway)
			.add_tag("method", format!("{:?}", self.method))
			.add_tag("uri", self.uri)
			.add_tag("id", format!("{}", self.id));

		let tcpip = self.tcpip;
		query.add_tag("src", tcpip.src.to_string())
			.add_tag("dst", tcpip.dst.to_string())
			.add_tag("sport", tcpip.sport)
			.add_tag("dport", tcpip.dport)
	}
}

impl PartialEq for Metrics {
	fn eq(&self, other: &Self) -> bool {
		self.time.eq(&other.time)
	}
}

impl Eq for Metrics {}

impl PartialOrd for Metrics {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.time.partial_cmp(&other.time)
	}
}

impl Ord for Metrics {
	fn cmp(&self, other: &Self) -> Ordering {
		self.time.cmp(&other.time)
	}
}

#[derive(Debug, Deserialize)]
enum Method {
	GET,
	PUT,
	POST,
	DELETE,
	OPTIONS,
	HEAD,
	TRACE,
	CONNECT,
	PATCH,
}

struct ExpiringNonceCache {
	expiring: HashSet<u128>,
	current: HashSet<u128>,
	creation: Instant,
	max_expiration: Duration,
}

impl ExpiringNonceCache {
	fn new(max_expiration: Duration) -> ExpiringNonceCache {
		ExpiringNonceCache {
			expiring: HashSet::new(),
			current: HashSet::new(),
			creation: Instant::now(),
			max_expiration,
		}
	}

	fn insert(&mut self, val: u128) -> bool {
		let now = Instant::now();
		if now.duration_since(self.creation) > self.max_expiration {
			let avg_len = (self.expiring.len() + self.current.len()) / 2;
			self.expiring = HashSet::with_capacity(avg_len);
			std::mem::swap(&mut self.expiring, &mut self.current);
		}

		self.creation = Instant::now();

		self.current.insert(val) || self.expiring.contains(&val)
	}
}

fn update_authenticator(assertion: Json<Assertion>, key: &hmacsha256::Key, nonces: &Mutex<ExpiringNonceCache>) -> Result<Metrics, AuthenticateError> {
	let sig = base64::decode(&assertion.sig)
		.map_err(|_| AuthenticateError::SignatureFormattingError)?;
	let sig = hmacsha256::Tag::from_slice(&sig)
		.ok_or(AuthenticateError::SignatureFormattingError)?;
	let message: Vec<_> = assertion.message.bytes().collect();

	if !hmacsha256::verify(&sig, &message, key) {
		return Err(AuthenticateError::InvalidSignature);
	}

	let message = &assertion.message;
	let message: Message = serde_json::from_str(message)
		.map_err(|_| AuthenticateError::InvalidMessage)?;

	let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("SystemTime is before UNIX EPOCH.");
	if now > message.expires {
		return Err(AuthenticateError::Expired);
	}
	if now + nonces.lock().unwrap().max_expiration < message.expires {
		return Err(AuthenticateError::Expired);
	}

	if !nonces.lock().unwrap().insert(message.nonce) {
		return Err(AuthenticateError::NonceReuse);
	}

	Ok(message.payload)
}

#[post("/update", format = "application/json", data = "<assertion>", rank = 1)]
fn update<'r>(assertion: Json<Assertion>, auth: State<Auth>, influx: State<InfluxClient>, minimum_wait: State<MinimumWait>) -> Result<Response<'r>, AuthenticateError> {
	let metrics = update_authenticator(assertion, &auth.key, &auth.nonces)?;

	let query = metrics.into_query();
	let result = influx.runtime.lock().unwrap().block_on(influx.client.query(&query));
	if result.is_err() {
		eprintln!("{}", result.unwrap_err());
	}

	Ok(build_response(Status::Ok, "Success.").finalize())
}

#[post("/update", rank = 2)]
fn update_bad_format<'r>() -> Response<'r> {
	build_response(Status::UnprocessableEntity, "Content-Type needs to be application/json.").finalize()
}

#[get("/update")]
fn update_get<'r>() -> Response<'r> {
	build_response(Status::MethodNotAllowed, "This end point is only available over POST.").finalize()
}

#[catch(400)]
fn bad_request<'r>() -> Response<'r> {
	build_response(Status::UnprocessableEntity, "The request could not be understood by the server due to malformed syntax.").finalize()
}

#[catch(404)]
fn not_found<'r>() -> Response<'r> {
	build_response(Status::NotFound, "Resource not found.").finalize()
}

#[catch(422)]
fn unprocessable_error<'r>() -> Response<'r> {
	build_response(Status::UnprocessableEntity, "Provided data was not formatted correctly.").finalize()
}

#[catch(500)]
fn internal_error<'r>() -> Response<'r> {
	build_response(Status::NotFound, "The server encountered and error.").finalize()
}

struct Auth { key: hmacsha256::Key, nonces: Mutex<ExpiringNonceCache> }

struct InfluxClient { client: Client, runtime: Mutex<Runtime> }

struct MinimumWait(f64);

pub fn start(key: hmacsha256::Key, max_expiration: Duration, influx_client: Client, influx_runtime: Runtime, minimum_wait: f64) -> LaunchError {
	let auth = Auth { key, nonces: Mutex::new(ExpiringNonceCache::new(max_expiration)) };
	let influx_client = InfluxClient { client: influx_client, runtime: Mutex::new(influx_runtime) };
	let minimum_wait = MinimumWait(minimum_wait);

	rocket::ignite()
		.mount("/", routes![update, update_get, update_bad_format])
		.register(catchers![not_found, internal_error, unprocessable_error, bad_request])
		.manage(auth)
		.manage(influx_client)
		.manage(minimum_wait)
		.launch()
}