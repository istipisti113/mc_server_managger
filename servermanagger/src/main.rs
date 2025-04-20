use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use std::{env, process, string};
use sqlx::FromRow;
use dotenv::dotenv;
use sqlx::Executor;
use std::process::{Command, Stdio};
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
//use async_process::Command as AsyncCommand;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[derive(Debug, FromRow)]
struct Server{
    id: i32,
    name: String,
    running: i32,
    port: i32,
    command: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let parameters : Vec<&str>= msg.content.split(" ").map(|a|a).collect();
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "ip" {
            let a = Command::new("curl")
                .arg("ifconfig.co")
                .output()
                .expect("curl failed");
            if let Err(why) = msg.channel_id.say(&ctx.http, String::from_utf8_lossy(&a.stdout) ).await {
                println!("Error sending message: {why:?}");
            }
        } else if parameters[0] == "list"{
            msg.channel_id.say(&ctx.http, list().await.join("\n")).await.unwrap();
        } else if parameters[0] == "add" {
            if msg.author.id != 609845831750778899 {
                msg.channel_id.say(&ctx.http, "nem vagy jogosult!").await.unwrap();
                return ;
            }
            let name = parameters[1];
            let running = parameters[2];
            let command = msg.content.split("\"").nth(1).expect("error in split");
            println!("{:?}, {:?}, {:?}", name, running, command);
            msg.channel_id.say(&ctx.http, addserver(name, running, command).await).await.expect("error sending message");
        } else if parameters[0] == "start" {
            let name = parameters[1];
            startserver(name).await;
            msg.channel_id.say(&ctx.http, String::from(name)+" started").await.unwrap();
        } else if parameters[0] == "stop" {
            let name = parameters[1];
            stopserver(name).await;
            msg.channel_id.say(&ctx.http, String::from(name)+" stopped").await.unwrap();
        } else if parameters[0] == "help" {
            msg.channel_id.say(&ctx.http, "!ping -- Pong!\nlist -- szerverlista\nstart <server> -- elinditja a <server> szervert\nstop <server> -- megallitja a <server> szervert\nadd <name> <running> <command> -- hozzaad egy uj szervert. Egy ember hasznalhatja es az en vagyok :P").await.unwrap();
        }
    }
}

async fn list() -> Vec<String>{
    let ip = Command::new("curl")
        .arg("ifconfig.co")
        .output()
        .expect("curl failed");
    let asd = MySqlConnectOptions::new()
        .username("asdf")
        .password("asdf")
        .database("teszt")
        .socket("/run/mysqld/mysqld.sock");
        // .host("localhost");
    let pool = MySqlPool::connect_with(asd).await.unwrap();
    // let response = sqlx::query("select * from mcservers;").execute(&pool).await.unwrap();
    let rows = sqlx::query_as::<_, Server>("select * from mcservers;").fetch_all(&pool).await.unwrap();
    let names : Vec<String>= rows.iter().map(|server|{
        let mut a = String::new();
        if server.running != 1{
            a = String::from("nem fut");
        } else {
            a=String::from(String::from("fut, ip: ")+&String::from_utf8_lossy(&ip.stdout).trim()+":"+&server.port.to_string());
        }
        server.name.clone() + " -- " + &a
    }).collect();
    println!("{:?}", names.join(", "));
    pool.close().await;
    return  names;
}

async fn addserver(name : &str, running: &str, command: &str ) -> String {
    let asd = MySqlConnectOptions::new()
        .username("asdf")
        .password("asdf")
        .database("teszt")
        .socket("/run/mysqld/mysqld.sock");
        // .host("localhost");
    let pool = MySqlPool::connect_with(asd).await.unwrap();
    let mut isrunning = 0;
    if running == "true" {
        isrunning = 1;
    }
    if let Err(why) = sqlx::query("insert into mcservers (name, running, command, port) values(?, ?, ?, 0)")
        .bind(name)
        .bind(isrunning)
        .bind(command)
        .fetch_all(&pool).await {
            println!("{:?}", why);
            return String::from("failed to add new server");
    } else {
        return String::from("new server added");
    }
}

async fn startserver(name : &str){
    let asd = MySqlConnectOptions::new()
        .username("asdf")
        .password("asdf")
        .database("teszt")
        .socket("/run/mysqld/mysqld.sock");
    let pool = MySqlPool::connect_with(asd).await.unwrap();
    //let id : i32 =sqlx::query_scalar("select id from mcservers where name = ?;").bind(name).fetch_one(&pool).await.unwrap();
    let ports : Vec<i32> = sqlx::query_scalar("select port from mcservers where running = 1;").fetch_all(&pool).await.unwrap();
    let mut actual_port = 25565;
    loop{
        if ports.contains(&actual_port){
            actual_port+=1;
        }
        else {
            break;
        }
    }
    let command :(String, String) =sqlx::query_as("select command, dir from mcservers where name = ?;").bind(name).fetch_one(&pool).await.unwrap();
    println!("{:?}", name);
    // process::Command::new("tmux").args(["-f", "/dev/null","new-session","-d","-s",name.trim()]).args(command.0.split(" ")).current_dir(command.1).spawn().unwrap();
    process::Command::new("tmux").current_dir(command.1).args(["-f","/dev/null","new-session","-d","-s",name
    ]).spawn().unwrap();
    process::Command::new("tmux").args(["send-keys","-t",&(String::from(name)+":0.0"),]).arg(command.0+" -port "+&actual_port.to_string()).arg("C-m").spawn().unwrap();
    sqlx::query("update mcservers set running = 1 where name = ?;").bind(name).execute(&pool).await.unwrap();
    sqlx::query("update mcservers set port = ? where name = ?;").bind(actual_port).bind(name).execute(&pool).await.unwrap();
    // let serverid : i32= sqlx::query_scalar("select id from mcservers where name =?;").bind(name).fetch_one(&pool).await.unwrap();
    // sqlx::query("insert into usedports (port) values (?);").bind(actual_port).execute(&pool).await.unwrap();
    // let portid : i32= sqlx::query_scalar("select id from usedports where port =?;").bind(actual_port).fetch_one(&pool).await.unwrap();
    // sqlx::query("insert into connector (serverid, portid) values (?,?);").bind(serverid).bind(portid).execute(&pool).await.unwrap();
}

async fn stopserver(name : &str) {
    let asd = MySqlConnectOptions::new()
        .username("asdf")
        .password("asdf")
        .database("teszt")
        .socket("/run/mysqld/mysqld.sock");
    let pool = MySqlPool::connect_with(asd).await.unwrap();
    process::Command::new("tmux").args(["send-keys","-t",&(String::from(name)+":0.0"),]).arg("C-m").spawn().unwrap();
    process::Command::new("tmux").args(["send-keys","-t",&(String::from(name)+":0.0"),]).arg("stop").arg("C-m").spawn().unwrap();
    sleep(Duration::from_secs(4));
    process::Command::new("tmux").args(["send-keys","-t",&(String::from(name)+":0.0"),]).arg("exit").arg("C-m").spawn().unwrap();
    sqlx::query("update mcservers set running = 0, port = 0 where name = ?;").bind(name).execute(&pool).await.unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Login with a bot token from the environment
    let token = env::var("TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
