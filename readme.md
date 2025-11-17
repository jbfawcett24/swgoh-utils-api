# Project Title (Update)

Pulls data from Star Wars: Galaxy of Heroes and returns it for use in my swgoh utils website


## Instructions for Build and Use

Steps to build and/or run the software:

1. Have Docker installed
2. Use docker compose to run all of the images within their own network
3. by default runs on port 7474

Instructions for using the software:

1. /characters - POST request with a charId key that is optional. if left blank it will return all of the characters. otherwise it will return the character based on its baseId
2. /account -  Uses JWT for authentication, returns information about the account in the JWT. pulls from the database if it is in there, otherwise get info from the game
3. /assets - static assets such as character thumbnails
4. /signIn - checks against database, returns a JWT if correct
5. /signUp - Creates a new user
6. /refreshAccount - syncs account data with game and puts into database

## Development Environment 

To recreate the development environment, you need the following software and/or libraries with the specified versions:

* vsCode for development
* Docker

## Useful Websites to Learn More

I found these websites useful in developing this software:

* [Rust Documentation](https://doc.rust-lang.org/stable/)
* [Rust Crate Docs](https://docs.rs/)

## Future Work

The following items I plan to fix, improve, and/or add to this project in the future:

- [ ] /guild endpoint -  returns information from the guild, such as all characters owned, tb results, members
- [X] better frontend - full website for all users to view the information
- [X] database - store account information and allow users to set goals with their account that is saved
