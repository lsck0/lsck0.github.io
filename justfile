default:
    @just --list

run:
    trunk serve --port 3000 --open

build:
    trunk build --release
