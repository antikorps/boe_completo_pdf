use std::process::exit;

pub struct FechaBoe {
    pub dia: usize,
    pub mes: usize,
    pub año: usize,
}

pub fn comprobar_formato(fecha: &str) -> FechaBoe {
    // DD-MM-AÑO
    let partes_fecha = fecha.trim().split("-").collect::<Vec<&str>>();
    if partes_fecha.len() != 3 {
        eprintln!("el formato de la fecha no es válido, se esperaba DD-MM-AÑO");
        exit(1);
    }
    let mut fecha_boe = FechaBoe {
        dia: 0,
        mes: 0,
        año: 0,
    };
    for (indice, valor) in partes_fecha.iter().enumerate() {
        let valor_parseado = match valor.parse::<usize>() {
            Err(_) => {
                eprintln!(
                    "el formato de la fecha no es válido, la parte {} no corresponde a una cifra",
                    valor
                );
                exit(1);
            }
            Ok(ok) => ok,
        };
        match indice {
            0 => {
                if valor_parseado > 31 {
                    eprintln!("el formato de la fecha no es válido, la cifra que corresponde al día {} no puede ser mayor que 31", valor_parseado);
                    exit(1);
                }
                fecha_boe.dia = valor_parseado;
            }
            1 => {
                if valor_parseado > 12 {
                    eprintln!("el formato de la fecha no es válido, la cifra que corresponde al mes {} no puede ser mayor que 12", valor_parseado);
                    exit(1);
                }
                fecha_boe.mes = valor_parseado;
            }
            2 => {
                if valor_parseado > 2050 || valor_parseado < 1950 {
                    eprintln!("el formato de la fecha no es válido, la cifra que corresponde al año {} debe ser un valor entre 1950 y 2050", valor_parseado);
                    exit(1);
                }
                fecha_boe.año = valor_parseado;
            }
            _ => unreachable!(),
        }
    }

    fecha_boe
}
