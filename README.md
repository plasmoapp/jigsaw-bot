<img src="https://i.imgur.com/0xKkOvz.png" width="128"/>

# Jigsaw Puzzle Bot

Turn any image into a Jigsaw Puzzle and solve it together with friends without leaving Telegram

Made with [Rust](https://www.rust-lang.org/) 🚀, [Godot](https://godotengine.org/) and [Redis](https://redis.io/)

Try it out now: [@jigsawpuzzlebot](https://t.me/jigsawpuzzlebot)

## Project overview

---

**[jigsaw-generator](./jigsaw-generator)**

- Made with Rust
- Subscribed for a [Redis PubSub](https://redis.io/docs/interact/pubsub/) request to generate a puzzle from an image
- Stores processed images in the file system and puzzle state in Redis
- Publishes a [Redis PubSub](https://redis.io/docs/interact/pubsub/) event when a puzzle is generated. Or when it fails to generate a puzzle

---

**[jigsaw-bot](./jigsaw-bot)**

- Made with [Teloxide](https://github.com/teloxide/teloxide) Rust framework
- An entry point of the Mini App
- Takes images from users and publishes a [Redis PubSub](https://redis.io/docs/interact/pubsub/) request to [jigsaw-generator](./jigsaw-generator) to generate a puzzle
- Notifies users when a puzzle finished or failed generating
- Allows users to share puzzles with friends using [Inline Query](https://core.telegram.org/bots/features#inline-requests)

---

**[jigsaw-game](./jigsaw-game)**

- Made with [Godot Engine](https://godotengine.org/)
- Frontend of the Mini App

---

**[jigsaw-backend](./jigsaw-backend)**

- Made with [Axum](https://github.com/tokio-rs/axum) Rust framework
- HTTP and WebSocket server
- Serves images generated by [jigsaw-generator](./jigsaw-generator)
- Serves [jigsaw-game](./jigsaw-game) exported as HTML
- Real-time multiplayer powered by WebSockets

---

**[jigsaw-common](./jigsaw-common)**

- Common Rust module shared between all other Rust modules

---

## Documentation

Most of the code has comments and is aproachable so if you're feeling courageous – clone the repo and jump straight into it

Here is also a list of specific features you might want to use in your project:

### Mini App authorization on WebSockets (Rust, Axum)

### Custom HTML export template for Godot

### Sync Godot Theme with Telegram Theme 

### 4

## How to run dev enviroment or deploy 

### 1. Download Docker and optionally ngrok

### 2. Create a Telegram Bot and a Web App 

### 3. Create .env file

### 4. Start the containers 

### Things to keep in mind

[jigsaw-game]() project is built inside of the [jigsaw-backend]() Dockerfile. After you've made changed to the [jigsaw-game]() project – you need to restart the [jigsaw-backend]() container
 
## Planned features

I've made some issues of things that I wanted to implement but didn't have time because of the contest deadline

Probably will work on them eventually but I also welcome contributions :)
