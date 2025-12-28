fmt:
    uv run --python 3.9 ruff check src
    uv run --python 3.9 ruff format --check src
    cargo fmt --all
    cargo clippy --all-targets --fix --allow-dirty --allow-staged --all-features
    leptosfmt am4-web/**/*.rs

check:
    uv run --python 3.9 ruff check src --fix
    uv run --python 3.9 ruff format src
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

start:
    uv run python -m src.am4 start api,bot

uninstall:
    uv pip uninstall am4

generate-stubs:
    cd src/am4/utils && uv run generate-stubs.py

install:
    uv pip install --reinstall --verbose ".[dev,bot,api,docs]" --config-settings=cmake.define.COPY_DATA=1

reinstall: uninstall install generate-stubs

test:
    uv run pytest
    cargo test --workspace --all-features --all-targets

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

build-docs:
    RUSTDOCFLAGS="-D rustdoc::all -A rustdoc::private-doc-tests" cargo doc --package am4 --all-features --no-deps

prepare-data:
    cd misc/scripts/prepare_data/ && cargo run

start-web:
    cd am4-web && trunk serve --release --minify

build-web:
    cd am4-web && trunk build --release --minify

dump params='':
    u dump -i docs -i src/am4/utils/tests -i src/am4/utils/stubs -i src/am4/utils/_official_api -i src/am4/db/pb_data/types.d.ts {{params}}