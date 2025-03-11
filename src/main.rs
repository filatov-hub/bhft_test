mod md;
mod mdd;
mod test_data;
mod feed;
mod utils;

use tokio::sync::{self, mpsc};
use std::{sync::{Arc, Mutex}, time::Duration};
use mdd::data::stream::{self, *};
use md::store::{quote::{BookSnapshot, Update}, *};
use feed::simulation::{self};

#[tokio::main]
async fn main() {

    println!(" -- start ");

    let (tx_ctrl, rx_ctrl)  = sync::mpsc::channel::<simulation::ControlCommand>(10);
    let (tx_msg, mut rx_msg) = mpsc::channel(100);
    
    let test_symbols = vec!["AAAA", "BBBB", "CCCC", "DDDD"];
    //let symbols = vec!["AAAA"];

    let quote_book = Arc::new(Mutex::new(quote::BookStore::new(5)));
    for s in &test_symbols {
        let _r = quote_book.lock().unwrap().create(s);
    }
    
    let generator = Arc::new(simulation::Generator::new(5, 2, test_symbols.clone()));
    generator.run(rx_ctrl, tx_msg).await;

    println!(" -- ControlCommand::RequestSnapshot");
    tx_ctrl.send(simulation::ControlCommand::RequestSnapshot).await.unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await; 
    println!(" -- ControlCommand::Start");
    tx_ctrl.send(simulation::ControlCommand::Start).await.unwrap();


    let (tx_json, mut rx_json) = mpsc::channel(100);
    tokio::spawn(async move {
        while let Some(msg) = rx_msg.recv().await {
            let json = convert_to_json(&msg);
            if let Err(_) = tx_json.send(json).await{
                eprintln!("Failed to send JSON data to storage");
            };
        }
    });

    let qb = quote_book.clone();
    tokio::spawn(async move {
        while let Some(json) = rx_json.recv().await {
            let mut quote_book = qb.lock().unwrap();
            let (symbol, income_data) = convert_from_json(json);
            match income_data {
                update::Data::Incremental(upd) => {
                    let _ = quote_book.update_quote(&symbol, upd);
                },
                update::Data::Snapshot(ss) => {
                    let _ = quote_book.update_snapshot(&symbol, ss);
                },
            }
        }
    });

    let symbols = test_symbols.clone();
    let qb = quote_book.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            let quote_book = qb.lock().unwrap();
            for s in &symbols{
                match quote_book.get_snapshot(s) {
                    None => {println!("unknown symbol: {s}!");},
                    Some(ss) => {
                        println!("\n symbol: {s}");
                        println!(" bids:");
                        for e in ss.bids {
                            println!("  {:?}", e);
                        }
                        println!(" asks:");
                        for e in ss.asks {
                            println!("  {:?}", e);
                        }
                    },
                }

            }
        }
    });



    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

}

fn convert_to_json(message: &simulation::Message) -> String {
    match message {
        simulation::Message::Quote(quote) => {
            let level = vec![QuoteLevel{price: quote.price.to_string(), volume: quote.volume.to_string()}];
            let (bids, asks) = match quote.side {
                quote::Side::Bid => (level, Vec::new()),
                quote::Side::Ask => (Vec::new(), level),
            };
            let quote_json = stream::UpdateResponse {
                event_type: serde_json::to_string(&quote.action).unwrap(),
                time_stamp: 1,
                symbol: quote.symbol.to_string(),
                first_update_id: 1,
                last_update_id: 1,
                bids: bids,
                asks: asks,
            };
            serde_json::to_string(&quote_json).unwrap()
        }
        simulation::Message::Snapshot(snapshot) => {
            let bids: Vec<QuoteLevel> = snapshot.prices.bids.iter()
                .map(|q| QuoteLevel{price: q.price.to_string(), volume: q.volume.to_string()}).collect();

            let asks: Vec<QuoteLevel> = snapshot.prices.asks.iter()
                .map(|q| QuoteLevel{price: q.price.to_string(), volume: q.volume.to_string()}).collect();

            let snapshot_json = stream::FullSnapshotResponse{
                id: "1".to_string(),
                symbol: Some(snapshot.symbol.to_string()),
                result: stream::Quotes{
                    last_update_id: 1,
                    bids,
                    asks,
                }
            };
            serde_json::to_string(&snapshot_json).unwrap()
        }
    }
}

fn convert_from_json(message: String) -> (String, update::Data) {
    let sr = serde_json::from_str::<stream::Response>(&message).unwrap();
    handle_stream_response(sr)

}

fn handle_stream_response(response: stream::Response) -> (String, update::Data) {
    match response {
        Response::FullSnapshot(sr) => {
            let bids: Vec<quote::Entry> = sr.result.bids.iter()
                .map(|ql| quote::Entry{price: ql.price.parse().unwrap(), volume: ql.volume.parse().unwrap()}).collect();

            let asks: Vec<quote::Entry> = sr.result.asks.iter()
                .map(|ql| quote::Entry{price: ql.price.parse().unwrap(), volume: ql.volume.parse().unwrap()}).collect();
            
            (sr.symbol.unwrap_or_default(), 
            update::Data::Snapshot(BookSnapshot{
                bids,
                asks,
            }))
        },
        Response::Update(ur) => {
            let side =  if ur.bids.is_empty() {quote::Side::Ask} else {quote::Side::Bid};
            let (price, volume) = match side {
                quote::Side::Bid => (ur.bids[0].price.parse::<f64>().unwrap(), ur.bids[0].volume.parse::<u32>().unwrap()),
                quote::Side::Ask => (ur.asks[0].price.parse::<f64>().unwrap(), ur.asks[0].volume.parse::<u32>().unwrap()),
            };
            (ur.symbol,
            update::Data::Incremental(Update {
                action: serde_json::from_str(&ur.event_type).unwrap(),
                side,
                price,
                volume,
            }))
        },
    }
    //
}





