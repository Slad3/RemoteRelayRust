# Remote Relay (Rust)

Originally coded in Python, Remote Relay is an http server interface that allows a user to control their in home smartplugs (and other forms of relays with a boolean state) externally from the respective smartplug's ecosystem.


## Getting Started

Only Rust is required to build and run from source.  

```bash
cargo build --release
cargo run --color=always
```


## Config and Config Options

As of currently the only supported way to configure the Rust version of this application is through a local
configuration file, though there are plans to add back configuration via a MongoDB server.

### Local Configuration

All local configuration is stored in `config.json` at root level of project directory.

An example configuration will look like this:
```json5
{
  "relays": [
    // TPLink Kasa Plug example relay configuration
    {
      "type": "KasaPlug",             // Type of relay, case-sensitive, required for configuration loader to differentiate
      "name": "Sample Name",          // Name of relay, use normal string restrictions
      "ip": "<ip address of relay>",  // IPv4 address of relay. Can get this from router devices
      "room": "bedroom"               // Optional room location of relay
    }
  ],
  "presets": [
    {
      "name": "Bedroom on",           // Name of relay, use normal string restrictions
      "enabled": true,                // For UI use, keeps the ability to store many different configurations without sending them all to a frontend
      "relays": {                     
        "Sample Name": true
      }
    }
  ]
}
```

As of this current version, by default presets will turn off every relay not explicitly stated to be turned on (set to `true`) in the preset config. Future efforts will be made toward an `explicit` boolean option per presets to let the user define if they want that preset to explicitly control all relays on preset toggle.
