
pub mod test_data{

pub const _FULL_SNAP : &str = r#"
{
  "id": "51e2affb-0aba-4821-ba75-f2625006eb43",
  "status": 200,
  "result": {
    "lastUpdateId": 10,
    "bids": [
      [
        "0.01379900",
        "3.43200000"
      ],
      [
        "0.01379800",
        "3.24300000"
      ]
    ],
    "asks": [
      [
        "0.01380000",
        "5.91700000"
      ],
      [
        "0.01380100",
        "6.01400000"
      ]
    ]
  },
  "rateLimits": [
    {
      "rateLimitType": "REQUEST_WEIGHT",
      "interval": "MINUTE",
      "intervalNum": 1,
      "limit": 6000,
      "count": 2
    }
  ]
}"#;

pub const _UPD : &str = r#"
{
  "e": "depthUpdate",
  "E": 1672515782136,
  "s": "BNBBTC",     
  "U": 157,          
  "u": 160,          
  "b": [             
    [
      "0.0024",      
      "10"           
    ]
  ],
  "a": [             
    [
      "0.0026",      
      "100"          
    ]
  ]
}"#;

}