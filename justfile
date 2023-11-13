build:
    cargo build
    mkdir -p assets/pluginlib
    mv target/debug/libbluefox_lib.a assets/pluginlib/
    mv target/debug/bluefox_game .

run:
    just build
    ./bluefox_game