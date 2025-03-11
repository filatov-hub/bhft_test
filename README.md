# bhft_test

**Тестовое задание реализует два основных компонента:**
1. Quote Book (`md::store::quote::BookStore`);
2. Quote Generator (`feed::simulation::Generator`);
----

`quote::BookStore` - простейшая имплементация посимвольного хранилища котировок L2. 
Методы:
- `new(depth: usize)`
	создает экземпляр, принимает глубину хранения цен ( market depth, bid/ask) для всех символов
	
- `create(&mut self, symbol: &str)`
	создает Book для хранения market depth (MD) конкретного символа
	
- `update_quote(&mut self, symbol: &str, update: Update)`
	апдейт MD по конкретному символу.
	`Update` - структура с `price, volume, side и update::Action (New, Change, Delete)`
	
- `update_snapshot(&mut self, symbol: &str, snapshot: BookSnapshot)`
	full snapshot по конкретному символу. Полностью заменяет стакан (MD) конкретного символа
	
- `get_snapshot(&self, symbol: &str) -> Option<BookSnapshot>`
	возвращает текущий snapshot по символу, два сортированных вектора bid/ask. верхушка - топ bid/ask соответственно

все методы thread safe.
	 
-------

`simulation::Generator` - генератор котировок L2
Методы:
- `new(depth: usize, new_from: usize, symbols: Vec<&'l str>)`
	создает экземпляр, принимает глубину хранения цен, new_from - аргумент задающий с какого уровня буду генерироваться новые котировки (`update::Action::New`), symbols - символы для последующей генерации котировок

	подробнее про `new_from`.

	Идея такая: по верхним уровням MD производится генерация котировок с `update::Action::Change`, т.е. у них меняется только volume, цена не меняется. А с уровня `new_from` производится генерация котировок с `update::Action::New` - меняются (генерируется) и цена и volume.
	Соответственно в BookStore получаем обновление для верхушки стакана, до `new_from` и замещение котировок ниже уровня `new_from`. 
	`update::Action::Delete` - не имплементирована в текущей реализации.
	
- `generate_quote(&self) -> Quote`
	генерация котировки L2 по случайному символу
	
- `generate_symbol_snapshot(&self, book: &Book) -> Snapshot`
	генерация snapshot для конкретного Book
	
оба метода приватные, вызываются из `simulation::Generator::run`

- `run(self: Arc<Self>, mut rx_ctrl: mpsc::Receiver<ControlCommand>, tx_msg: mpsc::Sender<Message>)`
	осуществляет генерацию котировок L2 по всем символам случайным образом, по таймеру (интервал захардкожен, нехорошо, да). Snapshot генерируется и отправляется только по запросу ControlCommand::RequestSnapshot
	принимает 2 канала. 
	1й - для управления генерацией: ControlCommand (Start, Stop, RequestSnapshot)
	2й - для передачи котировок
	
---

Разработка `simulation::Generator` вызвана необходимостью более стабильного процесса отладки, когда под контролем ценовой поток, тип апдейта итд. (думаю, знакомо).

Типы данных `simulation::Generator`'a и `quote::BookStore` не совпадают вполне намеренно. Т.е. Оба компонента полностью развязаны друг от друга. 

Хочу обратить внимание на то, как собственно происходит трансфер данных из генератора в `quote::BookStore`. Может вызвать недоумение, почему я делаю преобразование котировочных данных генератора в json (это как раз канал  `(tx_json, mut rx_json) = mpsc::channel(100))`, а потом делаю обратное преобразование из json в данные для апдейта `quote::BookStore`. Этим я показываю, что нет никакой связи (decoupling) между `simulation::Generator` и `quote::BookStore`. 

Идея такая, что simulation::Generator используется для отладки, а потом, в проде, заменяется на реальный источник котировок, который, к примеру, генерирует поток в виде json. 

Т.е. если бы котировочный поток был в другом представлении, то ни `simulation::Generator`, ни `quote::BookStore` менять бы не пришлось, нужно было бы заменить только конверторы. 

В качестве референса использовал примеры json с Binance (модуль test_data), _UPD и _FULL_SNAP соответственно. Примеры json сообщений в test_data.rs.
Json конверторы в `mdd::data::stream`. 

Реализация не без недостатков (например, `update:::Action::Delete` нет, а хардкод есть), но ее использование существенно упростило процесс отладки quote book. В дальнейшем, можно довести до ума.

---

Процесс (main.rs) В целом:
- создается экземпляр `quote::BookStore`
- задается набор тестовых символов. с которым он будет работать
- создается экземпляр `simulation::Generator` для того же набора символов
- запускается генератор
- генератору: команда `RequestSnapshot`, генерируются Snapshot'ы для всех символов, `quote::BookStore` их обрабатывает - готов к работе
- генератору: команда `Start`, начинается генерация инкрементальных `md::store::quote::Update`, `quote::BookStore` обновляется в соответствии с принятыми сообщениями
- последний spawn служит для периодического поллинга `quote::BookStore` для вывода на печать обновляемых стаканов (MD) посимвольно. Вывод вида:

```
 -- start
 -- ControlCommand::RequestSnapshot
 -- ControlCommand::Start

 symbol: AAAA
 bids:
  Entry { price: 4.7612, volume: 468 }
  Entry { price: 3.1503, volume: 219 }
  Entry { price: 2.7088, volume: 789 }
  Entry { price: 1.4618, volume: 135 }
  Entry { price: 1.3461, volume: 930 }
 asks:
  Entry { price: 16.4764, volume: 484 }
  Entry { price: 16.5907, volume: 334 }
  Entry { price: 16.6979, volume: 269 }
  Entry { price: 17.736, volume: 180 }
  Entry { price: 18.3308, volume: 887 }

 symbol: BBBB
 bids:
  Entry { price: 2.2185, volume: 297 }
  Entry { price: 1.5931, volume: 145 }
  Entry { price: 1.3622, volume: 725 }
  Entry { price: 1.2384, volume: 518 }
  Entry { price: 1.1516, volume: 320 }
 asks:
  Entry { price: 10.951, volume: 859 }
  Entry { price: 11.034, volume: 774 }
  Entry { price: 17.3915, volume: 897 }
  Entry { price: 19.3086, volume: 765 }
  Entry { price: 19.3178, volume: 227 }

--------------------

примерно 5-10 мин

--------------------
 
 symbol: AAAA
 bids:
  Entry { price: 4.7612, volume: 684 }
  Entry { price: 3.1503, volume: 569 }
  Entry { price: 3.1454, volume: 890 }
  Entry { price: 3.1405, volume: 166 }
  Entry { price: 3.1231, volume: 910 }
 asks:
  Entry { price: 16.4764, volume: 654 }
  Entry { price: 16.5907, volume: 952 }
  Entry { price: 16.6202, volume: 994 }
  Entry { price: 16.634, volume: 309 }
  Entry { price: 16.6429, volume: 717 }

 symbol: BBBB
 bids:
  Entry { price: 2.2185, volume: 495 }
  Entry { price: 1.5931, volume: 629 }
  Entry { price: 1.5907, volume: 980 }
  Entry { price: 1.5886, volume: 777 }
  Entry { price: 1.5878, volume: 513 }
 asks:
  Entry { price: 10.951, volume: 632 }
  Entry { price: 11.034, volume: 576 }
  Entry { price: 11.0542, volume: 366 }
  Entry { price: 11.0552, volume: 320 }
  Entry { price: 11.1574, volume: 215 }
```


-----------------------------------------


Хорошо видно, что верхушка стакана, 2 верхних уровня сохраняет значения цен (для bid/ask), меняется только volume, т.е. это `update::Action::Change`

Нижние 3 уровня меняют и цену и volume - это `update::Action::New`. Также видно, что нижние три уровня подтягиваются по цене к предыдущему уровню, так как худшие bid/ask постепенно вытесняются лучшими (верхние 2 уровня они вытеснить не могут, так как диапазон генерации намеренно ограничивается, иначе невозможно было бы реализовать постоянство для `update::Action::Change`). 

Так же по этой причине может произойти ошибка апдейта, что не является ошибкой, а просто говорит о том, что новая цена на `update::Action::New` совпала с имеющейся в стакане по данному символу. Безусловно это тоже можно отнести к недостаткам текущей реализации `simulation::Generator`'a

------------------

**Достоинства текущей реализации:**
- функциональная декомпозиция
- модульность 
- низкая связность
- относительная простота
- практически отсутствие клонирования (test_symbols.clone() можно не учитывать)
- асинхронность
- тест работает :)

**Недостатки:**
- хардкод некоторых параметров (стоило бы отдать на конфигурацию)
- практически отсутствие обработки ошибок,  думаю стоило больше закладываться на механизм error propagation 
- возможно стоило бы ввести трейты для используемых структур (насколько я понимаю, трейты можно рассматривать как некий аналог интерфейсов (чисто виртуальных классов)
- возможно стоило поменять структуры проекта, например: один модуль для одной структуры. Не уверен, не знаю еще как это принято в мире Rust
- возможно стоит поменять саму парадигму разработки от привычного ООП к более функциональному стилю
- пока еще не получается достигнуть такой простоты читаемости кода как я могу сделать это на c++

