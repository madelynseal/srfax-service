# SRFax Downloader Service
A basic service or daemon that checks an SRFax inbox and downloads any
available faxes to a specified directory

## Current State
- currently works but still has a few improvements to be made

## Build Requirements
- openssl (and pkg-config)
- probably a C compiler

## Configuration
### Main Config (config.json)
- `config.json` will be created if it does not exist, or you can use the flag
  `--write-config` to write the default config out
- the config should be laid out in a way that is self explanatory
    - `tick_rate` is in seconds
    - `email.server` does not support dns names, only ip:port

## SRFax Config (srfaxes.json)
- is an array of srfax configurations
- an example config is written if `srfaxes.json` does not exist
- the config should be laid out in a way that is self explanatory
- download_fmt supports `PDF` or `TIF`
- `--write-config` will not overwrite this file

## Install as Windows Service
- on windows, an `install` subcommand is available
- it will install srfax-service as a windows service with the name `SRFax`
- the name can be changed by changing `SERVICE_NAME` under `main.rs`
