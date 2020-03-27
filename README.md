# Rust Config Managment Tool

A rudimentary config managment tool for managing files and packages on debian based systems built in [Rust](https://www.rust-lang.org/).

``` ASCII
❯ ./rust-config-mgmt-tool --help
rust-config-mgmt-tool 0.1.0

USAGE:
    rust-config-mgmt-tool <configfile>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <configfile>
```

## Usage

### Configure

Configuration is to be provided to `rust-config-mgmt-tool` via a single config file in either `yaml` or `json` format.

The config file is made of two parts. A list of packages and a list of files.

At least one file and one package must be specified in the config file.

We recommend placing the config file and all file sources within their own directory for easier management.

E.g:

``` ASCII
rust-config-mgmt-tool
config
├── config.yml
└── files
    ├── index.php
    └── webserver.conf
```

#### Packages

This tool will ensure that specified packages are installed, and of a particular version if set.

Each package configuration within the `packages` section *must* contain *all* the following options:

* `name`

    The name of the package is defined when installed via: `apt-get install $name`

* `ensure`

    Either `latest` or a specific version of the package: E.g.: `1.4.6-1ubuntu3.9`

Example in `yaml`:

``` YAML
packages:
- name: nginx
  ensure: 1.4.6-1ubuntu3.9
- name: jq
  ensure: latest
```

Example in `json`:

``` JSON
{
  "packages": [
    {
      "ensure": "1.4.6-1ubuntu3.9",
      "name": "nginx"
    },
    {
      "ensure": "latest",
      "name": "jq"
    }
  ]
}
```

#### Files

This tool will ensure that the specified files are created in the location required, with the exact content and permissions as set in the config.

Additionally, creation or update of files can then trigger a service to be restarted.

Each file configuration within the `files` section *must* contain *all* the following options:

* `filename`

    This is the full path where the file will be created on machine to be configured.

    E.g.: `/etc/motd`

* `source`

    The relative path from the config file location for the content for the file.

    E.g.: `files/webserver.conf`

* `owner`

    The user that will own the file.

    E.g.: `root`

* `group`

    The group that will have access to the file.

    E.g.: `root`

* `mode`

    The Unix mode bits for the file. This is compatible with the [`chmod`](https://linux.die.net/man/1/chmod) linux utility.

    E.g.: `"0644"` = `-rw-r--r--`

    Read/Write for the owner, Read only for the group and everyone.

* `trigger`

    The name of the service to be restarted upon a file change or creation.

    If service no is to be restarted, this must be set to an empty string: `""`

    E.g.: `"nginx"` or `""`

Example in `yaml`:

``` YAML
files:
- filename: /etc/nginx/sites-available/default
  source: files/webserver.conf
  owner: root
  group: root
  mode: "0644"
  trigger: nginx
```

Example in `json`:

``` JSON
{
  "files": [
    {
      "source": "files/webserver.conf",
      "filename": "/etc/nginx/sites-available/default",
      "group": "root",
      "mode": "0644",
      "owner": "root",
      "trigger": "nginx"
    }
  ]
}
```

### Install

Copy onto the server via `scp`, `rsync` or similar both the `rust-config-mgmt-tool` binary and the `config` directory/files.

E.g.:

``` BASH
scp rust-config-mgmt-tool user@remote-host:~
scp -r config user@remote-host:~
```

### Run

`ssh` or login to the target server that you copied the `rust-config-mgmt-tool` and config files.

`root` or `sudo` access will be required to be able to install and manage packages.

Run the following command as `root` or `sudo`, replacing the config file location if needed with your correct location.

E.g.:

``` BASH
ssh user@remote-host
./rust-config-mgmt-tool config/config.yml
```

`rust-config-mgmt-tool` is idempotent, meaning that you can continue to safetly run the above command to ensure the server config continues match your specification.

### Example

The provided [example config](./config/config.yml) is expected to run on an Ubuntu 14.04 LTS server and will install and configure a webserver with a `Hello world!` php application.

Once the configuration is complete the server should be accessible on HTTP port 80: E.g.: `curl -sv "http://ADDRESS";`

Example output from a first run:

``` BASH
./rust-config-mgmt-tool config/config.yml
Checking server config to match: "/root/config/config.yml"
+ Package List Update
+ Package: nginx
+ Package: php5-fpm
+ File: "/etc/nginx/sites-available/default"
++ Updating file content: "/etc/nginx/sites-available/default"
+ File: "/var/hello-world-php/index.php"
++ Updating file content: "/var/hello-world-php/index.php"
+ Service: nginx
+ Service: php5-fpm
```

## Develop

### Dependencies

Please install the latest available version (or at least `1.42.0`) using `rustup` following the [recommended install instructions](https://www.rust-lang.org/tools/install).

This will install everything you need including the `rustc` compiler and the package/build manager `cargo`.

### Design

`rust-config-mgmt-tool` has been built in [Rust](https://www.rust-lang.org/) so as to create a single binary with no dependencies, allowing for easier deploys.

The config file supports both `yaml` and `json` formats for either human or machine easier parsing/readability depending on your preference.

Specification of the config file format is defined in a set of Structs at the top of [`main.rs`](./src/main.rs).

Currently the design in [`main.rs`](./src/main.rs) is a single function application. If any further extension of functionality is required, it would be recommended to split that function up into separate functions or for larger function sets, sub-libraries.

Certain functionality is achieved by shelling out to certain common linux cli tools, which are expected to be already available on the machine.

* `apt-get`
* `chown`
* `chmod`

### Building

Building the app requires the following steps:

``` BASH
cargo build
````

Result: `target/debug/rust-config-mgmt-tool`

Or for release optimised version:

``` BASH
cargo build --release
````

Result: `target/release/rust-config-mgmt-tool`

If you would like to build and run in one go:

``` BASH
cargo run
````

### Tests

The following tests assume you have `rustc`, `cargo` and `docker` installed.

The tests are running in an Ubuntu 14.04 LTS container as the target platform.

``` BASH
./run-tests.sh
```

### Known Issues

This is a **rudimentary** config managment tool. Below are some of the currently known issues.

* Packages are always installed before files

    It would be better to be able to set order/dependencies between files and packages

* Packages & Files are installed in the order specified in the config file.

    Like dependencies between files and packages, it would be preferable to be able to set dependencies between all resources.

* If a folder does not exist for a file, it will be auto created. However you are not able to specify the permissions & ownership of that folder.

* Package install/update actions are unable to trigger service restarts

* Services are restarted, not reloaded

* Service are not able to be defined as enabled or disabled and so follow the default settings within the package that installed them

* File permissions are enforced on every run, and do not trigger a service restart when they change

* Error messages could be clearer when the config file does not exactly match the format required

    E.g.: missing field

    ``` ASCII
    Error: Message("missing field `source`", Some(Pos { marker: Marker { index: 59, line: 6, col: 10 }, path: "files[0]" }))
    ```

* `apt-get update` is performed on every run. Ideally this would only be when the apt cache is stale to improve run times.

* there are no unit tests

* managing permissions on files requries shelling out to `chmod`. There is `rust` native ways to do this, but currently require much more work to keep the same usability (in the config file) instead of having to specify octals manually.
