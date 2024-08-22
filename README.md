# Remote Relay (Rust)

Originally coded in Python, Remote Relay is an http server interface that allows a user to control their in home smartplugs (and other forms of relays with a boolean state) externally from the respective smartplug's ecosystem.


## Getting Started

Only Rust is required to build and run from source.  

```bash
cargo build --release
cargo run --color=always
```


## Config and Config Options

Currently, you can set your configuration via local file in `config.json` or through a MongoDB database through a `.env` file. Application defaults to MongoDB config.

### Local Configuration

All local configuration is stored in `config.json` at root level of project directory.

An example configuration will look like this:
```json5
{
  "relays": [
    // TPLink Kasa Plug example relay configuration
    {
      "type": "KasaPlug",                                   // Type of relay, case-sensitive, required for configuration loader to differentiate
      "name": "Sample Name",                                // Name of relay, use normal string restrictions
      "ip": "<ip address of relay>",                        // IPv4 address of relay. Can get this from router devices
      "room": "bedroom"                                     // Optional room location of relay
    },
    {
      "type": "KasaMultiPlug",                              // Type of relay, case-sensitive, required for configuration loader to differentiate
      "names": ["Sample Name 1", "Sample Name 2"],          // Names of relays, length must match exactly with number of relays on device
      "ip": "<ip address of relay>",                        // IPv4 address of relay. Can get this from router devices
      "room": "bedroom"                                     // Optional room location of relay
    }
  ],
  "presets": [
    {
      "name": "Bedroom on",       // Name of relay, use normal string restrictions
      "enabled": true,            // For UI use, keeps the ability to store many different configurations without sending them all to a frontend
      "relays": {                     
        "Sample Name": true
      }
    }
  ]
}
```

As of this current version, by default presets will turn off every relay not explicitly stated to be turned on (set to `true`) in the preset config. Future efforts will be made toward an `explicit` boolean option per presets to let the user define if they want that preset to explicitly control all relays on preset toggle.


### Mongo Configuration

To set up a configuration through a Mongo Database, the following must be exact:
1. Your `.env` file must contain the connection string denoted by a `MONGODB_URL=` variable, placed at root level of running directory.
2. All collections must be in a database named `HomeConfig`
3. `Relays` and `Presets` (case-sensitive) collections must be present in `HomeConfig`
4. Relays and Presets are formatted just like in local config, with each relay/preset being its own document

For a visual representation:
  - `HomeConfig` (Database)
    - `Relays`   (Collection)
    - `Presets`  (Collection)

## Routes
### Index Routes
| Route    | Description                                                                                      |
|----------|--------------------------------------------------------------------------------------------------|
| /        | Health Check                                                                                     |
| /status  | Gets full status of all relays                                                                   |
| /refresh | Endpoint for refreshing config, useful for dynamic config loading testing and external debugging |

### Preset Routes
| Route                           | Description                                      |
|---------------------------------|--------------------------------------------------|
| /preset/getPresetNames          | Gets list of all preset names                    |
| /preset/setPreset/<preset_name> | Sets preset via name                             |

### Relay Routes
| Route                           | Description                                                                           |
|---------------------------------|---------------------------------------------------------------------------------------|
| /relay/<relay_name>/set/<value> | Gives command to specific relay. Commands include `ON`, `OFF`, `SWITCH`, and `STATUS` |


## Future Todos
- 
