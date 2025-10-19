# Project Title (Update)

Pulls data from Star Wars: Galaxy of Heroes and returns it for use in my swgoh utils website


## Instructions for Build and Use

Steps to build and/or run the software:

1. install rustup/cargo
2. run the swgoh-comlink and swgoh-ae2 docker images on ports 3000 and 3001 - https://github.com/swgoh-utils
3. '''cargo run''' to start the project
4. if you want the frontend, go to the frontent folder - '''npm run dev'''

Instructions for using the software:

1. /characters - POST request with a charId key that is optional. if left blank it will return all of the characters. otherwise it will return the character based on its baseId
2. /account -  POST request with an allyCode key. returns account information such as character levels, grand arena ranking, and guild
3. /assets - static assets such as character thumbnails

## Development Environment 

To recreate the development environment, you need the following software and/or libraries with the specified versions:

* see [Cargo.toml](./Cargo.toml) for libraries
* vsCode for development

## Useful Websites to Learn More

I found these websites useful in developing this software:

* [Rust Documentation](https://doc.rust-lang.org/stable/)
* [Rust Crate Docs](https://docs.rs/)

## Future Work

The following items I plan to fix, improve, and/or add to this project in the future:

* [ ] /guild endpoint -  returns information from the guild, such as all characters owned, tb results, members
* [ ] better frontend - full website for all users to view the information
* [ ] database - store account information and allow users to set goals with their account that is saved
