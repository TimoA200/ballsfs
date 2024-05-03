Link to the Repository: https://github.com/TimoA200/ballsfs

# Prerequisites

To build the project rust needs to be installed: https://www.rust-lang.org/tools/install

Docker must also be installed, if you want to use containers: https://docs.docker.com/engine/install/

# Building

(This step can be skipped, if the prebuild ubuntu binaries in `ubuntu-build` are used).
In order to create an optimized build of both the daemon and the client, simply run: `cargo build --release`. The binaries will be saved in `./target/release`.

# Installing

The BallsFS daemon can be installed either using docker, or without docker, this section will cover both methods.

## Non Docker

To install the BallsFS daemons simply put the `ballsd` binaries onto the servers, which are supposed to be part of the cluster.

Next configure the firewall, that the servers can reach each other using the port `8001`.

Next create the `balls-config.json` on each host and put it in the same folder as the `ballsd` binary.

The `balls-config.json` defines a list with the IPs of all the other peers, her is an example:

```json
{
  "peers": ["10.1.1.3", "10.1.1.4"]
}
```

Then simply run `./ballsd` on all the peers and the cluster should be running.

## Using Docker

To use ballsFS with docker, clone the whole project folder onto the different hosts, or at least all the important files, and folders.
Then create the `balls-config.json` for every host again, as described in the section above and put it into the root folder of the project.

Then simply run `docker compose up` on every host and the daemons should be running.

# Usage

You can use the `ballscli` binary to interact with the cluster.
It is important that you also create the `balls-config.json` file for the client in the root project of the folder.
It looks slightly different because it provides a list of daemons instead of peers, below is an example:

```json
{
  "daemons": ["10.1.1.2", "10.1.1.3", "10.1.1.4"]
}
```

Run `./ballscli --help` to get guidance of how to use the client.

Congrats🎉
