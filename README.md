# bevy_mine_sweeper
Mine sweeper game clone with rust and bevy engine

![image](https://user-images.githubusercontent.com/1101456/109294297-f6a67200-7867-11eb-80ba-dbe06aff9cd8.png)

cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./target --target web target/wasm32-unknown-unknown/debug/minesweeper.wasm