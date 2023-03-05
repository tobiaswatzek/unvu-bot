# unvu-bot

This is a Discord bot called Unvu written in Rust.
Its purpose is to amuse and support.

Development building and running can be done with the Rust toolchain
best installed via [rustup](https://rustup.rs/).
In order to start the bot a `.env` file has to be created that contains
valid values for the variables that can be seen in `.env.example`.

For production a Docker image can be built with the given `Dockerfile`.
To locally run the image and pass in the `.env` file the following
command can be used:

```txt
docker run --env-file=./.env unvu-bot
```
