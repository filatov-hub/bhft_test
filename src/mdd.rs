pub mod data{
    pub mod stream{
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(untagged)] // allows for multiple types in the same enum
        pub enum Response {
            FullSnapshot(FullSnapshotResponse),
            Update(UpdateResponse),
        }

        #[derive(Serialize, Debug)]
        pub struct SymbolSubscription<'l>{
            pub id: &'l str,
            pub method: &'l str,
            pub params: Params<'l>,
        }
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct SubscribeRequest<'l> {
            pub method: &'l str,
            pub params: Vec<&'l str>,
            pub id: u32,
        }
        #[derive(Serialize, Deserialize, Debug)]
        pub struct SymbolSubscription2<'l>{
            pub id: &'l str,
            pub method: &'l str,
            pub params: Symbols,
        }
        

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params<'l> {
            pub symbol: &'l str,
            pub limit: u32,
        }
    
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Symbols {
            pub symbols: Vec<String>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct FullSnapshotResponse {
            pub id: String,
            //pub status: u16,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub symbol: Option<String>,
            pub result: Quotes,
            // #[serde(rename = "rateLimits")]
            //pub rate_limits: Vec<RateLimit>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Quotes {
            #[serde(rename = "lastUpdateId")]
            pub last_update_id: u64,
            pub bids: Vec<QuoteLevel>,
            pub asks: Vec<QuoteLevel>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct QuoteLevel{
            pub price: String,
            pub volume: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        struct RateLimit {
            #[serde(rename = "rateLimitType")]
            rate_limit_type: String,
            interval: String,
            #[serde(rename = "intervalNum")]
            interval_num: u32,
            limit: u32,
            count: u32,
        }
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct UpdateResponse{
            #[serde(rename = "e")]
            pub event_type: String,

            #[serde(rename = "E")]
            pub time_stamp: u64,

            #[serde(rename = "s")]
            pub symbol: String,

            #[serde(rename = "U")]
            pub first_update_id: u64,

            #[serde(rename = "u")]
            pub last_update_id: u64,
            
            #[serde(rename = "b")]
            pub bids: Vec<QuoteLevel>,

            #[serde(rename = "a")]
            pub asks: Vec<QuoteLevel>,
        }

        // impl UpdateResponse {
        //     pub fn bid(&self) -> Option<&QuoteLevel> {
        //         self.bids.get(0)
        //     }
        //     pub fn ask(&self) -> Option<&QuoteLevel> {
        //         self.asks.get(0)
        //     }
        // }


    }
}