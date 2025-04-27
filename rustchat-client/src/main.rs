use std::net::{TcpStream, SocketAddr, TcpListener};
use std::io::{Error, ErrorKind}

use terminal_menu::*;

use chat_security::{SessionCryptData, Message};



fn run_server(socket_addr: SocketAddr) -> Result<SessionCryptData, Error>{

    let listener = TcpListener::bind(socket_addr)?;
    let mut stream = listener.accept()?.0;

    SessionCryptData::recieve_session(&mut stream)


}

fn connect_to_peer(ip: SocketAddr) -> Result<SessionCryptData, Error>{

    let mut stream = TcpStream::connect(ip)?;

    SessionCryptData::start_session(&mut stream)
}

fn try_display_listen() -> Result<SessionCryptData, Error> {

    let menu = menu(vec![string("Enter a port number to listen from: (leave blank to let rustchat choose)", "", true)]);
    run(&menu);
    let menu = mut_menu(&menu);
    let selection = menu.selection_value("Enter a port number to listen from: (leave blank to let rustchat choose)").trim();
    let selection_num: u16;
    if selection = "" { selection_num = 0; } 
    else { 
       if let Ok(parsed) != str::parse::<u16>(selection){
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid port number entered"));
       }
       else{
            selection_num = parsed;
       } 

    }




    


}


fn display_main_menu(label: &str){

    let menu = terminal_menu::menu(vec![
        label(label),
        button("Start listening for a connection"),
        button("Connect to a peer ip"),
        button("Exit")

    ]);

    run(&menu);

    let menu = mut_menu(&menu);
    let selection = menu.selected_item_name();

    match selection.trim(){

        "Start listening for a connection" => {
            
        },

        "Connect to a peer ip" => {
            

        },

        "Exit" => {
            println!("Exiting...");
            std::process::exit(0);

        }

        _ => {
            println!("Unexpected option, exiting...");
            std::process::exit(1);

        }

    }

}

fn main(){
    display_main_menu("Welcome to rustchat! Select an option: ");
}



