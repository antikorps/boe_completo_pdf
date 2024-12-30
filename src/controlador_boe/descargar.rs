use futures::future::join_all;
use lopdf::Document;
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::time::sleep;

use std::{collections::BTreeMap, env::{self, current_exe}, fs::File, io::{self, Write}, path::PathBuf, time::Duration};

use lopdf::{Bookmark, Object, ObjectId};

use super::{
    cliente_http,
    validar_fecha::{self, FechaBoe},
};

pub struct GestorDescargaBoe {
    pub cliente: Client,
    pub fecha: FechaBoe,
    pub enlaces_pdf: Vec<EnlacePDF>,
    pub pdf_memoria: Vec<Vec<u8>>,
    pub error_critico: Option<String>,
}
#[derive(Clone)]
pub struct EnlacePDF {
    pub apartado: String,
    pub url: String,
    pub titulo: String,
}

impl From<&str> for GestorDescargaBoe {
    fn from(f: &str) -> Self {
        let fecha = validar_fecha::comprobar_formato(f);
        println!("🟢 fecha incorporada válida");
        GestorDescargaBoe {
            cliente: cliente_http::nuevo_cliente_http(),
            fecha,
            enlaces_pdf: Vec::new(),
            pdf_memoria: Vec::new(),
            error_critico: None,
        }
    }
}

impl GestorDescargaBoe {
    fn crear_ruta_descarga(&self, sufijo: &str) -> PathBuf {
        let ruta_raiz = current_exe()
            .expect("no ha podido recuperarse la ruta del ejecutable")
            .parent()
            .expect("no ha podido recuperarse la ruta raiz del ejecutable")
            .to_owned();
        let nombre_archivo = format!(
            "{:02}_{:02}_{}_boe_completo{}",
            self.fecha.dia, self.fecha.mes, self.fecha.año, sufijo
        );
        let ruta_descarga = ruta_raiz.join(nombre_archivo);
        return ruta_descarga;
    }
    async fn buscar_pdf_disposiciones(&mut self) {
        let endpoint = format!(
            "https://boe.es/boe/dias/{}/{:02}/{:02}/",
            self.fecha.año, self.fecha.mes, self.fecha.dia
        );
        let html = match realizar_get_devolver_html_parseado(&endpoint, &self.cliente).await {
            Err(error) => {
                self.error_critico = Some(error);
                return;
            }
            Ok(ok) => ok,
        };
        let selector = Selector::parse("#indiceSumario .sumario .puntoPDF a")
            .expect("ha fallado el selector para los enlaces de las disposiciones y anuncios");
        let enlaces_coincidentes = devolver_coincidencias_enlace_pdf(
            html,
            &selector,
            String::from("Disposiciones y anuncios"),
        );
        for v in enlaces_coincidentes.clone() {
            self.enlaces_pdf.push(v);
        }
    }
    async fn buscar_pdf_notificaciones(&mut self) {
        if self.error_critico.is_some() {
            return;
        }
        // https://boe.es/boe_n/dias/2024/12/26/index.php?l=N
        let endpoint = format!(
            "https://boe.es/boe_n/dias/{}/{:02}/{:02}/index.php?l=N",
            self.fecha.año, self.fecha.mes, self.fecha.dia
        );
        let html = match realizar_get_devolver_html_parseado(&endpoint, &self.cliente).await {
            Err(error) => {
                self.error_critico = Some(error);
                return;
            }
            Ok(ok) => ok,
        };
        let selector = Selector::parse("#indiceSumarioN .sumario .puntoPDF a")
            .expect("ha fallado el selector para los enlaces de las notificaciones");
        let enlaces_coincidentes =
            devolver_coincidencias_enlace_pdf(html, &selector, String::from("Notificaciones"));
        for v in enlaces_coincidentes.clone() {
            self.enlaces_pdf.push(v);
        }
    }
    async fn buscar_pdf_edictos(&mut self) {
        if self.error_critico.is_some() {
            return;
        }
        // https://boe.es/boe_j/dias/2024/12/26/index.php?l=J
        let endpoint = format!(
            "https://boe.es/boe_j/dias/{}/{:02}/{:02}/index.php?l=J",
            self.fecha.año, self.fecha.mes, self.fecha.dia
        );
        let html = match realizar_get_devolver_html_parseado(&endpoint, &self.cliente).await {
            Err(error) => {
                self.error_critico = Some(error);
                return;
            }
            Ok(ok) => ok,
        };
        let selector = Selector::parse("#indiceSumarioN .sumario .puntoPDF a")
            .expect("ha fallado el selector para los enlaces de las notificaciones");
        let enlaces_coincidentes =
            devolver_coincidencias_enlace_pdf(html, &selector, String::from("Edictos Judiciales"));
        for v in enlaces_coincidentes {
            self.enlaces_pdf.push(v);
        }
    }
    fn generar_informe_descargas(&self) {
        if self.error_critico.is_some() {
            return;
        }
        let mut contenido = String::from("Apartado\tTítulo\tUrl");
        for e in &self.enlaces_pdf {
            let linea = format!("{}\t{}\t{}\n", e.apartado, e.titulo, e.url);
            contenido.push_str(&linea);
        }
        let ruta_tsv = self.crear_ruta_descarga("_informe.tsv");
        let mut archivo_tsv = match File::create(&ruta_tsv) {
            Err(error) => {
                eprintln!("🟡 no se ha podido crear el archivo para el informe {}", error);
                return;
            }
            Ok(ok) => ok,
        };
        match archivo_tsv.write_all(contenido.as_bytes()) {
            Err(error) => {
                eprintln!(
                    "🟡 ha fallado la escritura del archivo para el informe {}",
                    error
                );
                return;
            }
            Ok(_) => {
                println!("🟢 informe de descargas creado correctamente en {}", ruta_tsv.display());
            },
        }
    }
    async fn descargar_pdf_memoria(&mut self) {
        if self.error_critico.is_some() {
            return;
        }
        let total_archivos_descargar = self.enlaces_pdf.len();
        let mut archivos_descargados = 0;
        for lote in self.enlaces_pdf.chunks(1) {
            // Sleep por si las peticiones son demasiado seguidas
            sleep(Duration::from_secs(tiempo_espera_descargas())).await;
            let mut futuros = Vec::new();
            for enlace in lote {
                archivos_descargados += 1;
                print!("\r⏳ descargando archivo {:03} de {}", archivos_descargados, total_archivos_descargar);
                let _ = io::stdout().flush();
                futuros.push(realizar_get_devolver_bytes(&enlace.url, &self.cliente));
            }
            for r in join_all(futuros).await {
                match r {
                    Err(error) => {
                        eprintln!("{}", error)
                    }
                    Ok(ok) => {
                        self.pdf_memoria.push(ok);
                    }
                }
            }
        }
        println!("\n🟢 todos los archivos descargados en memoria")
    }
    fn unir_pdf_memoria(&mut self) {
        if self.error_critico.is_some() {
            return;
        }
        // Generate a stack of Documents to merge
        let mut documents = Vec::new();
        for pdf_data in &self.pdf_memoria {
            let pdf = Document::load_mem(&pdf_data).unwrap();
            documents.push(pdf);
        }
        // Define a starting max_id (will be used as start index for object_ids)
        let mut max_id = 1;
        let mut pagenum = 1;
        // Collect all Documents Objects grouped by a map
        let mut documents_pages = BTreeMap::new();
        let mut documents_objects = BTreeMap::new();
        let mut document = Document::with_version("1.5");

        for mut doc in documents {
            let mut first = false;
            doc.renumber_objects_with(max_id);

            max_id = doc.max_id + 1;

            documents_pages.extend(
                doc.get_pages()
                    .into_iter()
                    .map(|(_, object_id)| {
                        if !first {
                            let bookmark = Bookmark::new(
                                String::from(format!("Page_{}", pagenum)),
                                [0.0, 0.0, 1.0],
                                0,
                                object_id,
                            );
                            document.add_bookmark(bookmark, None);
                            first = true;
                            pagenum += 1;
                        }

                        (object_id, doc.get_object(object_id).unwrap().to_owned())
                    })
                    .collect::<BTreeMap<ObjectId, Object>>(),
            );
            documents_objects.extend(doc.objects);
        }

        // Catalog and Pages are mandatory
        let mut catalog_object: Option<(ObjectId, Object)> = None;
        let mut pages_object: Option<(ObjectId, Object)> = None;

        // Process all objects except "Page" type
        for (object_id, object) in documents_objects.iter() {
            // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
            // All other objects should be collected and inserted into the main Document
            match object.type_name().unwrap_or("") {
                "Catalog" => {
                    // Collect a first "Catalog" object and use it for the future "Pages"
                    catalog_object = Some((
                        if let Some((id, _)) = catalog_object {
                            id
                        } else {
                            *object_id
                        },
                        object.clone(),
                    ));
                }
                "Pages" => {
                    // Collect and update a first "Pages" object and use it for the future "Catalog"
                    // We have also to merge all dictionaries of the old and the new "Pages" object
                    if let Ok(dictionary) = object.as_dict() {
                        let mut dictionary = dictionary.clone();
                        if let Some((_, ref object)) = pages_object {
                            if let Ok(old_dictionary) = object.as_dict() {
                                dictionary.extend(old_dictionary);
                            }
                        }

                        pages_object = Some((
                            if let Some((id, _)) = pages_object {
                                id
                            } else {
                                *object_id
                            },
                            Object::Dictionary(dictionary),
                        ));
                    }
                }
                "Page" => {}     // Ignored, processed later and separately
                "Outlines" => {} // Ignored, not supported yet
                "Outline" => {}  // Ignored, not supported yet
                _ => {
                    document.objects.insert(*object_id, object.clone());
                }
            }
        }

        // If no "Pages" found abort
        if pages_object.is_none() {
            let mensaje_error =
                format!("ha fallado el proceso de unión de los PDF. Pages root not found.");
            self.error_critico = Some(mensaje_error);
            return;
        }

        // Iter over all "Page" and collect with the parent "Pages" created before
        for (object_id, object) in documents_pages.iter() {
            if let Ok(dictionary) = object.as_dict() {
                let mut dictionary = dictionary.clone();
                dictionary.set("Parent", pages_object.as_ref().unwrap().0);

                document
                    .objects
                    .insert(*object_id, Object::Dictionary(dictionary));
            }
        }

        // If no "Catalog" found abort
        if catalog_object.is_none() {
            let mensaje_error =
                format!("ha fallado el proceso de unión de los PDF. Catalog root not found.");
            self.error_critico = Some(mensaje_error);
            return;
        }

        let catalog_object = catalog_object.unwrap();
        let pages_object = pages_object.unwrap();

        // Build a new "Pages" with updated fields
        if let Ok(dictionary) = pages_object.1.as_dict() {
            let mut dictionary = dictionary.clone();

            // Set new pages count
            dictionary.set("Count", documents_pages.len() as u32);

            // Set new "Kids" list (collected from documents pages) for "Pages"
            dictionary.set(
                "Kids",
                documents_pages
                    .into_iter()
                    .map(|(object_id, _)| Object::Reference(object_id))
                    .collect::<Vec<_>>(),
            );

            document
                .objects
                .insert(pages_object.0, Object::Dictionary(dictionary));
        }

        // Build a new "Catalog" with updated fields
        if let Ok(dictionary) = catalog_object.1.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Pages", pages_object.0);
            dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

            document
                .objects
                .insert(catalog_object.0, Object::Dictionary(dictionary));
        }

        document.trailer.set("Root", catalog_object.0);

        // Update the max internal ID as wasn't updated before due to direct objects insertion
        document.max_id = document.objects.len() as u32;

        // Reorder all new Document objects
        document.renumber_objects();

        //Set any Bookmarks to the First child if they are not set to a page
        document.adjust_zero_pages();

        //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
        if let Some(n) = document.build_outline() {
            if let Ok(x) = document.get_object_mut(catalog_object.0) {
                if let Object::Dictionary(ref mut dict) = x {
                    dict.set("Outlines", Object::Reference(n));
                }
            }
        }

        document.compress();
        let ruta_pdf_guardado = self.crear_ruta_descarga(".pdf");
        match document.save(&ruta_pdf_guardado) {
            Err(error) => {
                let mensaje_error = format!("ha fallado el guardado del PDF unido {}", error);
                self.error_critico = Some(mensaje_error);
                return;
            }
            Ok(_) => {
                println!(
                    "🏆 boe completo generado y guardado en {}",
                    ruta_pdf_guardado.display()
                );
            }
        }
    }
    // fn testear_numero_reducido(&mut self) {
    //     let mut muestra_reducida = Vec::new();
    //     for (i, v) in self.enlaces_pdf.iter().enumerate() {
    //         if i == 3 {
    //             break;
    //         }
    //         let enlace = EnlacePDF {
    //             apartado: v.apartado.to_owned(),
    //             url: v.url.to_owned(),
    //             titulo: v.titulo.to_owned(),
    //         };
    //         muestra_reducida.push(enlace);
    //     }
    //     self.enlaces_pdf = muestra_reducida;
    // }
}

async fn realizar_get_devolver_html_parseado(url: &str, cliente: &Client) -> Result<Html, String> {
    let res = match cliente.get(url).send().await {
        Err(error) => {
            let mensaje_error = format!("ha fallado la petición a {} {} ", url, error);
            return Err(mensaje_error);
        }
        Ok(ok) => ok,
    };
    if !res.status().is_success() {
        let mensaje_error = format!(
            "la peticion a {} ha devuelto un status code no deseado {} ",
            url,
            res.status()
        );
        return Err(mensaje_error);
    }
    let html = match res.text().await {
        Err(error) => {
            let mensaje_error =
                format!("ha fallado la lectura de la respuesta a {} {} ", url, error);
            return Err(mensaje_error);
        }
        Ok(ok) => ok,
    };
    Ok(Html::parse_document(&html))
}

async fn realizar_get_devolver_bytes(url: &str, cliente: &Client) -> Result<Vec<u8>, String> {
    let res = match cliente.get(url).send().await {
        Err(error) => {
            let mensaje_error = format!("ha fallado la petición bytes a {} {} ", url, error);
            return Err(mensaje_error);
        }
        Ok(ok) => ok,
    };
    if !res.status().is_success() {
        let mensaje_error = format!(
            "la peticion bytes a {} ha devuelto un status code no deseado {} ",
            url,
            res.status()
        );
        return Err(mensaje_error);
    }
    let bytes = match res.bytes().await {
        Err(error) => {
            let mensaje_error = format!(
                "ha fallado la lectura en bytes de la respuesta a {} {} ",
                url, error
            );
            return Err(mensaje_error);
        }
        Ok(ok) => ok,
    };
    Ok(bytes.to_vec())
}
fn devolver_coincidencias_enlace_pdf(
    html: Html,
    selector: &Selector,
    apartado: String,
) -> Vec<EnlacePDF> {
    let mut enlaces = Vec::new();
    let coincidencias = html.select(&selector);
    for coincidencia in coincidencias {
        let href = coincidencia.attr("href");
        if href.is_none() {
            continue;
        }
        let url = href.unwrap().to_string();
        let titulo = coincidencia.text().collect::<String>();
        enlaces.push(EnlacePDF {
            url: format!("https://boe.es{}", url),
            titulo,
            apartado: apartado.to_owned(),
        });
    }
    if enlaces.len() == 0 {
            println!(
                "🟡 no se han encontrado enlaces {}", apartado,
            );
    } else {
        println!("🟢 {} enlaces encontrados en el apartado {}", enlaces.len(), apartado)
    }
    enlaces
}
fn tiempo_espera_descargas() -> u64{
    match env::var("BOE_COMPLETO_ESPERA") {
        Err(_) => {
            return 3;
        },
        Ok(var) => match var.parse::<u64>() {
            Err(_) => {
                return 3;
            }
            Ok(ok) => ok,
        }
    } 
}

pub async fn crear_gestor_descargas(fecha: &str) -> Result<(), String> {
    let mut gdb = GestorDescargaBoe::from(fecha);
    gdb.buscar_pdf_disposiciones().await;
    gdb.buscar_pdf_notificaciones().await;
    gdb.buscar_pdf_edictos().await;
    gdb.generar_informe_descargas();
    //gdb.testear_numero_reducido();
    gdb.descargar_pdf_memoria().await;
    gdb.unir_pdf_memoria();

    if gdb.error_critico.is_some() {
        return Err(gdb.error_critico.unwrap());
    }
    Ok(())
}
