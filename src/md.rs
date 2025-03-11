
pub mod store{

    pub mod update{
        use serde::{Deserialize, Serialize};
        use super::quote::{BookSnapshot, Update};
        

        #[derive(Debug, Serialize, Deserialize)]
        pub enum Action {
            New,
            Change,
            Delete,
        }

        pub enum Data{
            Incremental(Update),
            Snapshot(BookSnapshot),
        }
    }

    pub mod quote{
        use std::{cmp::Ordering, collections::{BTreeSet, HashMap}, sync::Mutex};
        use super::update;
        

        #[derive(Debug, PartialEq)]
        pub enum Side {
            Bid,
            Ask,
        }


        #[derive(Debug, Clone, PartialEq)]
        pub struct Entry {
            pub price: f64,
            pub volume: u32,
        }

        impl Entry {

            pub fn new(price: f64, volume: u32) -> Self {
                assert!(!price.is_nan(), "Price cannot be NaN");
                Entry { price, volume }
            }
        }

        impl Ord for Entry {
            fn cmp(&self, other: &Self) -> Ordering {
                self.price.partial_cmp(&other.price).unwrap()
            }
        }
        
        impl PartialOrd for Entry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Eq for Entry {}


        #[derive(Debug)]
        pub struct BookSnapshot{
            pub bids: Vec<Entry>,
            pub asks: Vec<Entry>,
        }
        
        impl BookSnapshot {
            
            pub fn new()-> BookSnapshot{
                BookSnapshot{
                    bids: Vec::new(),
                    asks: Vec::new(),
                }
            }
        }

        struct Book {
            bids: BTreeSet<Entry>,  
            asks: BTreeSet<Entry>,
            depth: usize,
        }
        
        impl Book {

            fn new(depth: usize) -> Self {
                Book {
                    bids: BTreeSet::new(),
                    asks: BTreeSet::new(),
                    depth,
                }
            }
        
            fn update(&mut self, update: Update)  {
                let  prices: &mut BTreeSet<Entry> =  match update.side {
                    Side::Ask => &mut self.asks,
                    Side::Bid=> &mut self.bids,
                };
                let entry = Entry::new(update.price, update.volume);
                let res = match update.action {
                    update::Action::New => prices.insert(entry),
                    update::Action::Change => prices.replace(entry).is_some(),
                    update::Action::Delete => prices.remove(&entry),
                };
                if !res {
                    eprintln!("Failed to update {:?}", update);
                }
                if self.bids.len() > self.depth{
                    self.bids.pop_first();
                }
                if self.asks.len() > self.depth{
                    self.asks.pop_last();
                }
            }

        }

        #[derive(Debug)]
        pub struct Update{
            pub action: update::Action,
            pub side: Side,
            pub price: f64,
            pub volume: u32,
        }


        pub struct BookStore{
            books: Mutex<HashMap<String, Book>>,
            depth: usize,
        }

        impl BookStore {

            pub fn new(depth: usize) -> Self {
                BookStore {
                    books: Mutex::new(HashMap::new()),
                    depth,
                }
            }

            pub fn create(&mut self, symbol: &str)-> Result<(), String> {
                let mut books = self.books.lock().unwrap();
                match books.insert(symbol.to_string(), Book::new(self.depth)){
                    Some(_) => Ok(()),
                    None => Err(format!("Book with {symbol} already exist")),
                }
            }

            pub fn update_quote(&mut self, symbol: &str, update: Update)-> Result<(), String> {
                let mut books = self.books.lock().unwrap();
                match books.get_mut( symbol) {
                    Some(book) => { book.update(update); Ok(())  },
                    None => Err(format!("{symbol} is not found!")),
                }
            }

            pub fn update_snapshot(&mut self, symbol: &str, snapshot: BookSnapshot)-> Result<(), String> {
                let mut books = self.books.lock().unwrap();
                match books.get_mut( symbol ) {
                    Some(book) => { 
                        book.bids = snapshot.bids.into_iter().collect();
                        book.asks = snapshot.asks.into_iter().collect();
                        Ok(()) },
                    None => Err(format!("{symbol} is not found!")),
                }
            }

            pub fn get_snapshot(&self, symbol: &str) -> Option<BookSnapshot>{
                let books = self.books.lock().unwrap();
                match books.get(symbol) {
                    Some(book) => Some(BookSnapshot{
                        bids: book.bids.iter().rev().cloned().collect(),
                        asks: book.asks.iter().cloned().collect(),
                    }),
                    None => None,
                }
            }
                
        }

    }
}