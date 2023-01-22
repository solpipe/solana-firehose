The `solana-geyser-plugin-firehose` crate implements a plugin streaming
account data over Grpc using the [Plugin Framework](https://docs.solana.com/developing/plugins/geyser-plugins).  The proto files are in the `./proto` directory.  This crate was forked from the postgres guide `solana-geyser-plugin-postgres`.

### Configuration File Format

The plugin is configured using the input configuration file. An example
configuration file looks like the following:

```
{
	"libpath": "/solana/target/release/libsolana_geyser_plugin_firehose.so",
	"grpc_listen_url": "localhost:50051",
	"threads": 20,
	"batch_size": 20,
	"panic_on_db_errors": true,
	"accounts_selector" : {
		"accounts" : ["*"]
	}
}
```
The `grpc_listen_url` is mainly the port on which the grpc server will listen.

TODO: change `panic_on_db_errors` to work with the tonic Grpc server.


### Account Selection

The `accounts_selector` can be used to filter the accounts that should be persisted.

For example, one can use the following to persist only the accounts with particular
Base58-encoded Pubkeys,

```
    "accounts_selector" : {
         "accounts" : ["pubkey-1", "pubkey-2", ..., "pubkey-n"],
    }
```

Or use the following to select accounts with certain program owners:

```
    "accounts_selector" : {
         "owners" : ["pubkey-owner-1", "pubkey-owner-2", ..., "pubkey-owner-m"],
    }
```

To select all accounts, use the wildcard character (*):

```
    "accounts_selector" : {
         "accounts" : ["*"],
    }
```

### Transaction Selection

`transaction_selector`, controls if and what transactions to store.
If this field is missing, none of the transactions are stored.

For example, one can use the following to select only the transactions
referencing accounts with particular Base58-encoded Pubkeys,

```
"transaction_selector" : {
    "mentions" : \["pubkey-1", "pubkey-2", ..., "pubkey-n"\],
}
```

The `mentions` field supports wildcards to select all transaction or
all 'vote' transactions. For example, to select all transactions:

```
"transaction_selector" : {
    "mentions" : \["*"\],
}
```

To select all vote transactions:

```
"transaction_selector" : {
    "mentions" : \["all_votes"\],
}
```



### Performance Considerations

This plugin has not been benchmarked yet.