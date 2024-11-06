# `posql_db`

Example demonstrating an implementation of a simple csv-backed database with Proof of SQL capabilities.

## Install
Run `cargo install --example posql_db --path crates/proof-of-sql` to install the example.

> [!NOTE]
> To run this example without the `blitzar` (i.e CPU only )feature 
> ```bash
> cargo install --example posql_db --path crates/proof-of-sql --no-default-features --features="rayon"
> ```

## Quick Start Exmaple
Run the following
```bash
posql_db create -t sxt.table -c a,b -d BIGINT,VARCHAR
posql_db append -t sxt.table -f hello_world.csv
posql_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
posql_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
```