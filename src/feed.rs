
pub mod simulation{
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use rand::Rng;
    use tokio::select;
    use tokio::time::interval;
    use tokio::sync::mpsc;
    use crate::md::store::*;
    use crate::md::store::quote::{self, Side};
    use crate::utils::math;

    struct Book{
        symbol: Arc<String>,
        bids: Vec<f64>,
        asks: Vec<f64>
    }

    impl<'l> Book {
        fn new(symbol: &'l str) -> Self {
            Book {
                symbol: Arc::new(symbol.to_string()),
                bids: Vec::new(),
                asks: Vec::new(),
            }
        }
    }

    pub struct Quote{
        pub action: update::Action,
        pub symbol: Arc<String>,
        pub side: quote::Side,
        pub price: f64,
        pub volume: u32,
    }

    pub struct Snapshot{
        pub symbol: Arc<String>,
        pub prices: quote::BookSnapshot,
    }
    
    #[derive(Debug)]
    pub enum ControlCommand {
        Start,
        Stop,
        RequestSnapshot,
    }

    pub enum Message {
        Quote(Quote),
        Snapshot(Snapshot),
    }


    pub struct Generator{
        //rx_ctrl: Arc<Mutex<tokio::sync::mpsc::Receiver<ControlCommand>>>,
        const_price_books: Vec<Book>,
        depth: usize,
        new_from: usize,
    }

    impl<'l> Generator {
        pub fn new(depth: usize, new_from: usize, symbols: Vec<&'l str>) -> Self{
            let books = Self::generate_books(new_from, &symbols);
            Generator{
                //rx_ctrl: Arc::new(Mutex::new(rx_ctrl)),
                depth,
                const_price_books: books,
                new_from,
            }
        }

        fn generate_books(new_from: usize, symbols: &Vec<&'l str>) ->  Vec<Book>{           
            let mut books = Vec::new();
            for s in symbols {
                books.push(Self::generate_symbol_book(s, new_from));
            }
            books
        } 

        fn generate_symbol_book(symbol: &'l str, new_from: usize) ->  Book{
            let mut book = Book::new(symbol);
            let mut rng = rand::rng();
            for _i in 0 .. new_from {
                book.bids.push(math::round(rng.random_range(1.0..=10.0), 4));
                book.asks.push(math::round(rng.random_range(10.0..=20.0),4));
            }
            sort(&mut book.bids);
            sort(&mut book.asks);
            book.bids.reverse();
            book
        }

        fn generate_quote(&self) -> Quote {
            let mut rng = rand::rng();
            let book_idx = rng.random_range(0.. self.const_price_books.len());
            let book = &self.const_price_books[book_idx];
            let side = if rng.random_range(0..=1) == 0 {Side::Bid} else {Side::Ask};
            let price;
            let action_idx = rng.random_range(0..2); // without delete
            let action = match action_idx {
                0 => {
                    price =  match side {
                        Side::Bid => math::round(rng.random_range(1.0..book.bids[self.new_from - 1]), 4),
                        Side::Ask => math::round(rng.random_range(book.asks[self.new_from - 1]..=20.0), 4),
                    };
                    update::Action::New
                },
                _ => {
                    let price_idx = rng.random_range(0.. self.new_from);
                    price = match side {
                        Side::Bid => book.bids[price_idx],
                        Side::Ask => book.asks[price_idx],
                    };
                    match action_idx {
                        1 => update::Action::Change,
                        _ => update:: Action::Delete,
                    }
                },
            };
            Quote{
                action,
                symbol: book.symbol.clone(),
                side,
                price,
                volume: match action_idx {
                    2 => 0,
                    _ => rng.random_range(100..1000),
                } 
            }
        } 

        fn generate_symbol_snapshot(&self, book: &Book) -> Snapshot {
            let mut rng = rand::rng();
            let mut prices =  quote::BookSnapshot::new();

            for i in 0..self.depth {
                let volume = rng.random_range(100..1000);
                let bid = if i < self.new_from {
                    book.bids[i]
                } 
                else {
                    math::round(rng.random_range(1.0..book.bids[self.new_from - 1]), 4)
                };
                prices.bids.push(quote::Entry::new(bid, volume));

                let volume = rng.random_range(100..1000);
                let ask = if i < self.new_from {
                    book.asks[i]
                } 
                else {
                    math::round(rng.random_range(book.asks[self.new_from - 1]..=20.0), 4)
                };
                prices.asks.push(quote::Entry::new(ask, volume));
            }
            sort(&mut prices.bids);
            prices.bids.reverse();
            sort(&mut prices.asks);
            Snapshot{
                symbol: book.symbol.clone(),
                prices,
            }
        }

        pub async fn run(self: Arc<Self>, mut rx_ctrl: mpsc::Receiver<ControlCommand>, tx_msg: mpsc::Sender<Message>){
            let mut interval = interval(Duration::from_millis(40));
            let mut is_running = false;
            let this = Arc::clone(&self);
            tokio::spawn(async move {
                loop {
                    select! {
                        _ = interval.tick() => {
                            if is_running{
                                let q = this.generate_quote();
                                if let Err(_) = tx_msg.send(Message::Quote(q)).await {
                                    eprintln!("Failed to send quote");
                                }
                            }
                        }
                        Some(cmd) = rx_ctrl.recv() => {
                            match cmd {
                                ControlCommand::Start => { is_running = true; },
                                ControlCommand::Stop => { is_running = false; },
                                ControlCommand::RequestSnapshot => {
                                    for (i, book) in this.const_price_books.iter().enumerate() {
                                        let ss = this.generate_symbol_snapshot(book);
                                        if let Err(_) = tx_msg.send(Message::Snapshot(ss)).await {
                                            eprintln!("Failed to send snapshot");
                                        }
                                        if i % 10 == 0 {
                                            tokio::task::yield_now().await;
                                        }
                                    }
                                }
                            }
                        }   
                    }
                }
            });

        }

    }

    fn sort<T: PartialOrd>(values: &mut Vec<T>) {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
    
}

