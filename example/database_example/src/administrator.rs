
use rocket::{Request, Outcome};
use rocket::request::FromRequest;
use std::collections::HashMap;
// use rocket::{Request, Data, Outcome, Response};
// use rocket::http::{Cookie, Cookies, MediaType, ContentType, Status, RawStr};
// use rocket::request::{FlashMessage, Form, FromRequest,FromForm, FormItems, FromFormValue, FromParam};
// use rocket::response::{content, NamedFile, Redirect, Flash, Responder, Content};
// use rocket::response::content::Html;

use super::PGCONN;
use auth::authorization::*;
// use auth::sanitization::*;

/// The AdministratorCookie type is used to indicate a user has logged in as an administrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministratorCookie {
    pub userid: u32,
    pub username: String,
    pub display: Option<String>,
}

/// The AdministratorForm type is used to process a user attempting to login as an administrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministratorForm {
    pub username: String,
    pub password: String,
}

impl CookieId for AdministratorCookie {
    fn cookie_id<'a>() -> &'a str {
        "acid"
    }
}

impl CookieId for AdministratorForm {
    fn cookie_id<'a>() -> &'a str {
        "acid"
    }
} 

impl AuthorizeCookie for AdministratorCookie {
    // type CookieType = AdministratorCookie;
    
    /// The store_cookie() method should contain code that
    /// converts the specified data structure into a string
    /// 
    /// This is likely to be achieved using one of the serde
    /// serialization crates.  Personally I would use either
    /// serde_json or serde's messagepack implementation ( rmp-serde [rmps]).
    /// 
    /// Json is portable and human readable.  
    /// 
    /// MsgPack is a binary format, and while not human readable is more
    /// compact and efficient.
    fn store_cookie(&self) -> String {
        // String::from("This is my cooky")
        // let ser = ::serde_json::to_string(self).expect("Could not serialize");
        ::serde_json::to_string(self).expect("Could not serialize")
    }
    
    
    /// The retrieve_cookie() method deserializes a string
    /// into a cookie data type.
    /// 
    /// Again, serde is likely to be used here.
    /// Either the messagepack or json formats would work well here.
    /// 
    /// Json is portable and human readable.  
    /// 
    /// MsgPack is a binary format, and while not human readable is more
    /// compact and efficient.
    #[allow(unused_variables)]
    // fn retrieve_cookie(string: String) -> Option<Self::CookieType> {
    fn retrieve_cookie(string: String) -> Option<Self> {
        let mut des_buf = string.clone();
        let des: Result<AdministratorCookie, _> = ::serde_json::from_str(&mut des_buf);
        if let Ok(cooky) = des {
            Some(cooky)
        } else {
            None
        }
        // Some(
        //     AdministratorCookie {
        //         userid: 66,
        //         username: "andrew".to_string(),
        //         display: Some("Andrew Prindle".to_string()),
        //     }
        // )
    }
}

impl AuthorizeForm for AdministratorForm {
    type CookieType = AdministratorCookie;
    
    /// Authenticate the credentials inside the login form
    fn authenticate(&self) -> Result<Self::CookieType, AuthFail> {
        let conn = PGCONN.lock().unwrap();
        let qrystr = format!("SELECT userid, username, display FROM users WHERE username = '{}' AND password = '{}' AND is_admin = '1'", &self.username, &self.password);
        let is_user_qrystr = format!("SELECT userid FROM users WHERE username = '{}'", &self.username);
        let is_admin_qrystr = format!("SELECT userid FROM users WHERE username = '{}' AND is_admin = '1'", &self.username);
        let password_qrystr = format!("SELECT userid FROM users WHERE username = '{}' AND password = '{}'", &self.username, &self.password);
        println!("Attempting query: {}", qrystr);
        if let Ok(qry) = conn.query(&qrystr, &[]) {
            if !qry.is_empty() && qry.len() == 1 {
                let row = qry.get(0);
                
                let display_opt = row.get_opt(2);
                let display = match display_opt {
                    Some(Ok(d)) => Some(d),
                    _ => None,
                };
                
                return Ok(AdministratorCookie {
                    userid: row.get(0),
                    username: row.get(1),
                    display,
                });
            }
        }
        // let username_qry = conn.query(&is_user_qrystr, &[])
        // match conn.query(&is_user_qrystr, &[]) {
        //     Ok(q) => return Err(AuthFail::new(self.username.clone(), "Username was not found.".to_string()));,
        //     Err(_) => return Err(AuthFail::new(self.username.clone(), "Username was not found.".to_string()));,
        //     _ => {},
        // }
        // if username_qry.is_err() {
        if let Ok(eqry) = conn.query(&is_user_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Username was not found.".to_string()));
            }
        }
        if let Ok(eqry) = conn.query(&is_admin_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                // In production this message may be more harmful than useful as it
                // would be able to tell anyone who is an administrator and thus the
                // message should be changed to something like Unkown error or Invalid username/password
                return Err(AuthFail::new(self.username.clone(), "User does not have administrator priveleges.".to_string()));
            }
        }
        if let Ok(eqry) = conn.query(&password_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Invalid username / password combination.".to_string()));
            }
        }
        Err(AuthFail::new(self.username.clone(), "Unkown error..".to_string()))
        
        // println!("Authenticating {} with password: {}", &self.username, &self.password);
        // if &self.username == "administrator" && &self.password != "" {
        //     Ok(
        //         AdministratorCookie {
        //             userid: 1,
        //             username: "administrator".to_string(),
        //             display: Some("Administrator".to_string()),
        //         }
        //     )
        // } else {
        //     Err(
        //         AuthFail::new(self.username.to_string(), "Incorrect username".to_string())
        //     )
        // }
    }
    
    /// Create a new login form instance
    fn new_form(user: &str, pass: &str, _extras: Option<HashMap<String, String>>) -> Self {
        AdministratorForm {
            username: user.to_string(),
            password: pass.to_string(),
        }
    }
    
}

impl<'a, 'r> FromRequest<'a, 'r> for AdministratorCookie {
    type Error = ();
    
    /// The from_request inside the file defining the custom data types
    /// enables the type to be checked directly in a route as a request guard
    /// 
    /// This is not needed but highly recommended.  Otherwise you would need to use:
    /// 
    /// `#[get("/protected")] fn admin_page(admin: AuthCont<AdministratorCookie>)`
    /// 
    /// instead of:
    /// 
    /// `#[get("/protected")] fn admin_page(admin: AdministratorCookie)`
    fn from_request(request: &'a Request<'r>) -> ::rocket::request::Outcome<AdministratorCookie,Self::Error>{
        let cid = AdministratorCookie::cookie_id();
        let mut cookies = request.cookies();
        
        match cookies.get_private(cid) {
            Some(cookie) => {
                if let Some(cookie_deserialized) = AdministratorCookie::retrieve_cookie(cookie.value().to_string()) {
                    Outcome::Success(
                        cookie_deserialized
                    )
                } else {
                    Outcome::Forward(())
                }
            },
            None => Outcome::Forward(())
        }
    }
}
