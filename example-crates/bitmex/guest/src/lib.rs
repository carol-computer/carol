use carol_guest::*;
pub use time;

#[derive(
    Clone, Copy, Debug, serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode,
)]
pub enum Index {
    BXBT,
}

#[derive(Debug, Clone, Copy, bincode::Encode, bincode::Decode)]
pub struct AttestIndexPrice {
    pub price: u64,
    pub signature: BlsSignature,
}

#[derive(Clone, Copy, Debug, bincode::Encode, bincode::Decode)]
pub struct OffsetDateTime(#[bincode(with_serde)] pub time::OffsetDateTime);

#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct AttestMessage {
    pub price: u64,
    pub time: OffsetDateTime,
    pub index: Index,
}

set_contract!(BitMexAttest);

#[derive(bincode::Decode, bincode::Encode)]
pub struct BitMexAttest {
    pub index: Index,
}

#[carol_contract]
impl BitMexAttest {
    pub fn attest_to_price_at_minute(
        &self,
        time: OffsetDateTime,
    ) -> Result<AttestIndexPrice, String> {
        let price = self.index_price_at_minute(time)?;
        let message = AttestMessage {
            price,
            time,
            index: self.index,
        };

        let encoded_message =
            bincode::encode_to_vec(&message, bincode::config::standard()).unwrap();

        let signature = global::bls_static_sign(&encoded_message);

        Ok(AttestIndexPrice { price, signature })
    }

    pub fn index_price_at_minute(&self, time: OffsetDateTime) -> Result<u64, String> {
        let time = time.0;
        let mut url =
            url::Url::parse("https://www.bitmex.com/api/v1/instrument/compositeIndex").unwrap();
        let symbol = match self.index {
            Index::BXBT => ".BXBT",
        };
        #[derive(serde::Serialize)]
        struct Filter<'a> {
            symbol: &'a str,
            #[serde(rename = "timestamp.hh")]
            timestamp_hour: u8,
            #[serde(rename = "timestamp.uu")]
            timestamp_min: u8,
            #[serde(rename = "timestamp.date")]
            timestamp_date: time::Date,
            #[serde(rename = "timestamp.ss")]
            timestamp_second: u8,
        }

        #[derive(serde::Deserialize, Debug, Clone, Copy)]
        #[serde(rename_all = "camelCase")]
        struct Price {
            last_price: f64,
        }
        let filter = serde_json::to_string(&Filter {
            timestamp_date: time.date(),
            timestamp_hour: time.hour(),
            timestamp_min: time.minute(),
            timestamp_second: 0,
            symbol,
        })
        .expect("serializes correctly");
        url.query_pairs_mut()
            .append_pair("symbol", symbol) // only interested in index
            .append_pair("filter", &filter)
            .append_pair("columns", "lastPrice,timestamp"); // only necessary fields

        let response = http::http_get(url.as_str());
        let price_at_time = serde_json::from_slice::<[Price; 1]>(&response.body).unwrap()[0];
        Ok(price_at_time.last_price.floor() as u64)
    }
}
