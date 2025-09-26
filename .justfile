list:
    just --list

game *ARGS:
    cargo run -- {{ARGS}}

client *ARGS:
    cargo run -- --connect ws://127.0.0.1:1155 {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}
