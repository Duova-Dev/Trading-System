// interfacing code for streamr



pub mod interface {
    use url::Url;
    use tungstenite::{connect, Message};

    pub fn test_tungstenite(address: &str) {

        let (mut socket, response) = 
            connect(Url::parse(address).unwrap()).expect("Can't connect.");
        
        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");

        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }

        socket
            .write_message(Message::Text(r#"
            {
                "method": "SUBSCRIBE",
                "params": [
                  "btcusdt@trade"
                ],
                "id": 1
              }
            "#.into()))
            .unwrap();
        loop {
            let msg = socket.read_message().expect("Error reading message");
            println!("Received: {}", msg);
        }
    }
}
