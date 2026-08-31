//! Que una migración nueva obligue a recompilar.
//!
//! `sqlx::migrate!("./migrations")` mete el contenido del directorio **dentro
//! del binario** al compilar. Pero Cargo no sabe que ese directorio es una
//! entrada: solo vigila los `.rs`. Así que añadir una migración y reconstruir
//! dejaba el binario igual, con la migración fuera, y **sin un solo aviso**:
//! `cargo build` decía «Finished», la API arrancaba, y `_sqlx_migrations` seguía
//! teniendo las filas de antes.
//!
//! El fallo no se ve hasta que algo consulta una columna que no existe, y para
//! entonces parece un error del código y no del build.
//!
//! Una línea lo arregla, y no hay forma de que se olvide.

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
