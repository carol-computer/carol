use carol_guest::*;
pub use time;

#[derive(
    Debug, Clone, Copy, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize,
)]
pub struct AttestIndexPrice<S> {
    pub price: u64,
    pub signature: S,
}

#[derive(
    Clone, Copy, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize,
)]
pub struct OffsetDateTime(
    #[bincode(with_serde)]
    #[serde(with = "time::serde::iso8601")]
    pub time::OffsetDateTime,
);

#[derive(Clone, Debug, bincode::Encode)]
pub struct AttestMessage {
    pub time: OffsetDateTime,
    pub price: u64,
    pub symbol: String,
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct BitMexAttest;

#[derive(bincode::Encode)]
pub struct AttestBit<'a> {
    pub symbol: &'a str,
    pub time: OffsetDateTime,
    pub n_bits: u8,
    pub bit_index: u8,
    pub bit_value: bool,
}

#[carol]
impl BitMexAttest {
    /// Like [`Self::attest_to_price_at_minute`] but provides a BLS signature for every bit of the
    /// price capped to `2^n_bits - 1`. This is for use in *[Cryptographic Oracle-Based Conditional
    /// Payments]* like schemes.
    ///
    /// [Cryptographic Oracle-Based Conditional Payments]: https://eprint.iacr.org/2022/499
    #[activate]
    pub fn bit_decompose_attest_to_price_at_minute(
        &self,
        cap: &(impl cap::Bls + cap::Http),
        time: OffsetDateTime,
        symbol: String,
        n_bits: u8,
    ) -> Result<AttestIndexPrice<Vec<bls::Signature>>, String> {
        let price = self.index_price_at_minute(cap, &symbol, time)?;

        let capped_price = price.min((1 << n_bits) - 1);
        let signatures = (0..(n_bits as usize)).map(|bit_index| {
            let bit_value = ((capped_price >> bit_index) & 0x01) == 1;
            let attest_bit = AttestBit {
                symbol: &symbol,
                time,
                n_bits,
                bit_index: bit_index as u8,
                bit_value,
            };
            let message = bincode::encode_to_vec(attest_bit, bincode::config::standard()).unwrap();
            cap.bls_static_sign(&message)
        });

        Ok(AttestIndexPrice {
            price,
            signature: signatures.collect(),
        })
    }

    /// Provide a single BLS signature over rounded down price of `symbol` at the minute at
    /// described at `time` (seconds are ignored).
    #[activate]
    pub fn attest_to_price_at_minute(
        &self,
        cap: &(impl cap::Bls + cap::Http),
        time: OffsetDateTime,
        symbol: String,
    ) -> Result<AttestIndexPrice<bls::Signature>, String> {
        let price = self.index_price_at_minute(cap, &symbol, time)?;
        let message = AttestMessage {
            price,
            time,
            symbol,
        };

        let encoded_message = bincode::encode_to_vec(message, bincode::config::standard()).unwrap();

        let signature = cap.bls_static_sign(&encoded_message);

        Ok(AttestIndexPrice { price, signature })
    }

    pub fn index_price_at_minute(
        &self,
        cap: &impl cap::Http,
        symbol: &str,
        time: OffsetDateTime,
    ) -> Result<u64, String> {
        let time = time.0;
        let mut url =
            url::Url::parse("https://www.bitmex.com/api/v1/instrument/compositeIndex").unwrap();

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

        let response = cap.http_get(url.as_str());
        let price_at_time = serde_json::from_slice::<[Price; 1]>(&response.body).unwrap()[0];
        Ok(price_at_time.last_price.floor() as u64)
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "network_tests")]
    #[test]
    fn index_price_at_minute() {
        use crate::{BitMexAttest, OffsetDateTime};
        use carol_guest::TestCap;
        let time = OffsetDateTime(time::macros::datetime!(2023-04-15 0:00 UTC));
        let cap = TestCap::default();
        let index_price = BitMexAttest
            .attest_to_price_at_minute(&cap, time, ".BXBT".into())
            .unwrap();
        assert_eq!(index_price.price, 30492);
    }
}
