cargo test  ${*-} 2>&1 | sed 's@'"$(pwd)"'@@g' | sed 's@'"$HOME"'/@@g' | sed 's@'".cargo/registry/src/index.crates.io-"'@@g'
