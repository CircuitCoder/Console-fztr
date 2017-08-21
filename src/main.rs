#[macro_use]
extern crate lazy_static;

extern crate iron;

#[macro_use]
extern crate router;

extern crate wkhtmltopdf;

extern crate rand;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use router::Router;

use std::fs::File;
use std::path::Path;

use std::io::Read;
use std::io::Write;

use rand::Rng;

lazy_static!{
    static ref ARGS: Vec<String> = std::env::args().collect::<Vec<String>>();
    static ref INDEX: String = {
        let mut index = String::new();
        let mut indexfile = File::open(&ARGS[2]).unwrap();
        indexfile.read_to_string(&mut index).unwrap();
        index
    };
    static ref STORAGE: Box<Path> = Path::new(&ARGS[1]).into();
}

fn main() {
    let mut router = Router::new();

    router.get("/", |req: &mut Request| -> IronResult<Response> {
        let id = rand::thread_rng().gen_ascii_chars().take(20).collect::<String>();
        let u = url_for!(req, "frontend", "id" => id);
        return Ok(Response::with((status::Found, Redirect(u))));
    }, "index");

    router.get("/:id", |_: &mut Request| -> IronResult<Response> {
        Ok(Response::with((status::Ok, INDEX.clone())))
    }, "frontend");

    router.get("/content/:id", |req: &mut Request| -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>().unwrap().find("id");
        if query.is_none() {
            return Ok(Response::with((status::NotFound)));
        };

        let query = query.unwrap();

        let mut data = String::new();
        let file = File::open(STORAGE.join(query));
        if file.is_err() {
            return Ok(Response::with((status::NotFound)));
        }
        let mut file = file.unwrap();

        let r = file.read_to_string(&mut data);
        if r.is_err() {
            Ok(Response::with((status::Ok, "")))
        } else {
            Ok(Response::with((status::Ok, data)))
        }
    }, "fetch");

    router.post("/content/:id", |req: &mut Request| -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>().unwrap().find("id");
        if query.is_none() {
            return Ok(Response::with((status::NotFound)));
        };

        let query = query.unwrap();

        let mut data = Vec::new();
        req.body.read_to_end(&mut data)
            .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading request")))?;

        let mut file = File::create(STORAGE.join(query)).map_err(|e| IronError::new(e, (status::InternalServerError, "Error creating the file")))?;
        file.write_all(&data).map_err(|e| IronError::new(e, (status::InternalServerError, "Error writing the file")))?;

        Ok(Response::with((status::Ok)))
    }, "update");

    router.get("/rendered/:id", |req: &mut Request| -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>().unwrap().find("id");
        if query.is_none() {
            return Ok(Response::with((status::NotFound)));
        };

        let query = query.unwrap();

        let mut p = STORAGE.join(query);
        p.set_extension("pdf");

        // TODO: optimize
        if p.exists() {
            let mut result = String::new();
            File::open(p)
                .unwrap()
                .read_to_string(&mut result)
                .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading the file")))?;
            return Ok(Response::with((status::Ok, result)));
        };

        // TODO: rebuild
        return Ok(Response::with((status::NotFound)));
    }, "render");

    println!("Starting server at {}", &ARGS[3]);
    Iron::new(router).http(&ARGS[3]).unwrap();
}
