build:
    cargo build
    mkdir -p assets/pluginlib
    mv target/debug/libinternal_bluefox_lib.a assets/pluginlib/
    mv target/debug/bluefox_game .

run:
    just build
    ./bluefox_game