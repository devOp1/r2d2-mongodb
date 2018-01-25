extern crate r2d2;
extern crate mongodb;
extern crate backtrace;

use std::fmt;
use std::default::Default;
use std::error;
use std::error::Error as _StdError;
use mongodb::{ThreadedClient, Client};
use mongodb::db::ThreadedDatabase;

use backtrace::Backtrace;

pub const ADMIN_DB_NAME: &'static str = "admin";

#[derive(Debug)]
pub enum Error {
    Other(mongodb::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.description(), self.cause().unwrap())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Other(ref err) => err.cause()
        }
    }
}

#[derive(Default)]
pub struct MongodbConnectionManager {
    host: Option<String>,
    port: u16,
    uri: Option<String>,
    password: Option<String>,
}

impl MongodbConnectionManager {
    pub fn new(host: &str, port: u16) 
            -> Result<MongodbConnectionManager, mongodb::error::Error> {
        Ok(MongodbConnectionManager {
            host: Some(host.to_owned()),
            port: port,
            ..Default::default()
        })
    }   

    pub fn from_uri(uri: &str) -> Result<MongodbConnectionManager, mongodb::error::Error> {
        
        Ok(MongodbConnectionManager {
            uri: Some(uri.to_owned()),
            ..Default::default()
        }) 

    }

    // if you have a uri unfriendly password
    pub fn from_uri_password(uri: &str, password: &str) -> Result<MongodbConnectionManager, mongodb::error::Error> {
        
        Ok(MongodbConnectionManager {
            uri: Some(uri.to_owned()),
            password: Some(password.to_owned()),
            ..Default::default()
        }) 

    }
}

impl r2d2::ManageConnection for MongodbConnectionManager {
    type Connection = Client;
    type Error = Error;

    fn connect(&self) -> Result<Client, Error> {
        if let Some(ref host) = self.host {
            Client::connect(host, self.port).map_err(|err| Error::Other(err))
        } else if let Some (ref uri) = self.uri {
            let result = Client::with_uri(uri).map_err(|err| Error::Other(err));
            let client: Client = result.unwrap();
            let cs = mongodb::connstring::parse(uri).unwrap();
            let password;
            if let Some(ref uri_unfriendly_password) = self.password {
               password = uri_unfriendly_password;
            } else {
                password = &cs.password.as_ref().unwrap();
            }
            let adb = client.db(ADMIN_DB_NAME);
            adb.auth(&cs.user.unwrap(),&password).expect("need username/password");
            println!("new authentication completed: {:?}\n bt (not an error): {:?}", uri,Backtrace::new());
            Ok(client)
        } else {
            Err(Error::Other(mongodb::error::Error::DefaultError("db host and uri not set".to_owned())))
        }
        
    }

    fn is_valid(&self, _conn: &mut Client) -> Result<(), Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Client) -> bool {
        false
    }
}
