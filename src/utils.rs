use crate::serenity::prelude::TypeMapKey;

use reqwest::Client as HttpClient;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
	type Value = HttpClient;
}