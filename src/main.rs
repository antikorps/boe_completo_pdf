use std::{env::args, process::exit};

mod controlador_boe;
#[tokio::main]
async fn main() {
    let argumentos = args().collect::<Vec<String>>();
    match controlador_boe::descargar::crear_gestor_descargas(&argumentos[argumentos.len() - 1]).await {
        Ok(_) => exit(0),
        Err(_) => exit(1),
    }
}
