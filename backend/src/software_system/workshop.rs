//! Physical inventory and frozen loadouts. All inventory mutations serialize per owner.
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use crate::{auth::Ctx, error::ApiError, state::AppState};
use super::dtos::Stats;

pub const LEVELS: [i32; 6] = [1, 3, 6, 10, 15, 21];
pub fn slots(level: i32) -> i32 { LEVELS.iter().filter(|&&l| l <= level.max(1)).count() as i32 }
pub fn scale(value: i32, tier: usize) -> i32 {
    (value as f64 * [1.0, 1.35, 1.8, 2.4, 3.2][tier]).round() as i32
}
fn bad(s: &str) -> ApiError { ApiError::BadRequest(s.into()) }

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub allowed_capabilities: Vec<String>,
    pub allowed_slots: Vec<i32>,
    pub min_level: i32,
    pub max_copies: i32,
    pub excludes: Vec<String>,
    pub grants: Vec<String>,
    pub active: bool,
    pub collectable: bool,
    pub deltas: Stats,
}
#[derive(Deserialize)]
struct Template { id: Option<i64>, code: String, name: String, config: Config }
#[derive(Deserialize)]
struct Equip { software_id: i64, slot: i32, module_id: i64, tier: i32 }
#[derive(Deserialize)]
struct Remove { software_id: i64, slot: i32 }
#[derive(Deserialize)]
struct Fuse { module_id: i64, tier: i32 }

pub fn configure(c: &mut web::ServiceConfig) {
    c.route("/software-system/workshop", web::get().to(inventory))
        .route("/software-system/workshop/equip", web::post().to(equip))
        .route("/software-system/workshop/unequip", web::post().to(unequip))
        .route("/software-system/workshop/fuse", web::post().to(fuse));
}
pub fn configure_admin(c: &mut web::ServiceConfig) {
    c.route("/software-modules", web::get().to(catalog))
        .route("/software-modules", web::post().to(save_template));
}
pub async fn functions(pool: &PgPool, id: i64) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT program_functions($1)").bind(id).fetch_one(pool).await
}
async fn inventory(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    let catalog: Vec<Value> = sqlx::query_scalar("SELECT jsonb_build_object('id',id,'code',code,'name',name,'config',workshop_config) FROM research_modules ORDER BY id").fetch_all(&state.pool).await?;
    let stock: Vec<Value> = sqlx::query_scalar("SELECT to_jsonb(s) - 'user_id' FROM module_stock s WHERE user_id=$1 AND quantity>0 ORDER BY module_id,tier").bind(ctx.id).fetch_all(&state.pool).await?;
    let equipped: Vec<Value> = sqlx::query_scalar("SELECT to_jsonb(m) - 'owner_id' FROM program_modules m JOIN software s ON s.id=m.software_id WHERE m.owner_id=$1 AND s.user_id=$1 AND NOT s.is_npc AND NOT s.soft_hidden ORDER BY software_id,slot").bind(ctx.id).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(json!({"catalog":catalog,"stock":stock,"equipped":equipped,"slot_levels":LEVELS,"grades":["Estándar","Optimizado","Experimental","Clasificado","Zero-day"]})))
}
async fn fuse(state: web::Data<AppState>, ctx: Ctx, input: web::Json<Fuse>) -> Result<HttpResponse, ApiError> {
    if !(0..4).contains(&input.tier) { return Err(bad("Este grado no admite fusión.")); }
    let mut tx=state.pool.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE").bind(ctx.id).fetch_one(&mut *tx).await?;
    let n=sqlx::query("UPDATE module_stock SET quantity=quantity-3 WHERE user_id=$1 AND module_id=$2 AND tier=$3 AND quantity>=3").bind(ctx.id).bind(input.module_id).bind(input.tier).execute(&mut *tx).await?.rows_affected();
    if n==0 { return Err(bad("Necesitas 3 copias libres del mismo módulo y grado.")); }
    sqlx::query("INSERT INTO module_stock VALUES($1,$2,$3,1) ON CONFLICT(user_id,module_id,tier) DO UPDATE SET quantity=module_stock.quantity+1").bind(ctx.id).bind(input.module_id).bind(input.tier+1).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(json!({"tier":input.tier+1})))
}
async fn lock_program(tx: &mut sqlx::Transaction<'_,sqlx::Postgres>, user:i64, id:i64) -> Result<(i32,String,i64),ApiError> {
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE").bind(user).fetch_one(&mut **tx).await?;
    let row=sqlx::query("SELECT s.soft_version,c.code,s.server_id FROM software s JOIN software_profiles p ON p.software_id=s.id JOIN software_capabilities c ON c.id=p.capability_id WHERE s.id=$1 AND s.user_id=$2 AND NOT s.is_npc AND NOT s.soft_hidden FOR UPDATE OF s").bind(id).bind(user).fetch_optional(&mut **tx).await?.ok_or(ApiError::NotFound)?;
    let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_running WHERE soft_id=$1) OR EXISTS(SELECT 1 FROM processes WHERE soft_id=$1 AND completed_at IS NULL)").bind(id).fetch_one(&mut **tx).await?;
    if busy { return Err(bad("Descarga el programa de memoria y espera a que terminen sus procesos antes de cambiar módulos.")); }
    Ok((row.get(0),row.get(1),row.get(2)))
}
async fn sync_size(tx: &mut sqlx::Transaction<'_,sqlx::Postgres>, id:i64, server:i64) -> Result<(),ApiError> {
    sqlx::query("SELECT server_id FROM hardware WHERE server_id=$1 FOR UPDATE").bind(server).fetch_one(&mut **tx).await?;
    let fits:bool=sqlx::query_scalar("SELECT p.weight <= h.hdd-COALESCE((SELECT sum(soft_size) FROM software WHERE server_id=$2 AND id<>$1),0) FROM effective_software_profiles p JOIN hardware h ON h.server_id=$2 WHERE p.software_id=$1").bind(id).bind(server).fetch_one(&mut **tx).await?;
    if !fits { return Err(bad("No queda espacio en el disco para esta configuración.")); }
    sqlx::query("UPDATE software s SET soft_size=p.weight,soft_ram=p.ram_need FROM effective_software_profiles p WHERE s.id=$1 AND p.software_id=s.id").bind(id).execute(&mut **tx).await?;
    Ok(())
}
async fn equip(state: web::Data<AppState>, ctx: Ctx, input: web::Json<Equip>) -> Result<HttpResponse,ApiError> {
    if !(0..=4).contains(&input.tier) { return Err(bad("Grado inválido.")); }
    let mut tx=state.pool.begin().await?;
    let (level,cap,server)=lock_program(&mut tx,ctx.id,input.software_id).await?;
    if input.slot<1 || input.slot>slots(level) { return Err(bad("Ranura bloqueada: investiga el programa para subir su nivel.")); }
    let row=sqlx::query("SELECT code,name,workshop_config FROM research_modules WHERE id=$1 FOR SHARE").bind(input.module_id).fetch_optional(&mut *tx).await?.ok_or(ApiError::NotFound)?;
    let code:String=row.get(0);
    let config:Config=serde_json::from_value(row.get(2)).map_err(|_|bad("Configuración de módulo inválida."))?;
    if !config.active || level<config.min_level || !config.allowed_slots.contains(&input.slot) || (!config.allowed_capabilities.is_empty() && !config.allowed_capabilities.contains(&cap)) { return Err(bad("Este módulo no admite esta capacidad, nivel o posición de ranura.")); }
    let existing:Vec<(i32,i64,Value)>=sqlx::query_as("SELECT slot,module_id,snapshot FROM program_modules WHERE software_id=$1").bind(input.software_id).fetch_all(&mut *tx).await?;
    if existing.iter().any(|m|m.0==input.slot) { return Err(bad("Retira primero el módulo de esta ranura.")); }
    if existing.iter().filter(|m|m.1==input.module_id).count()>=config.max_copies as usize { return Err(bad("Se alcanzó el máximo de copias de este módulo por programa.")); }
    for (_,_,s) in &existing {
        if config.excludes.iter().any(|x|s["code"].as_str()==Some(x.as_str())) || s["excludes"].as_array().is_some_and(|xs|xs.iter().any(|x|x.as_str()==Some(&code))) { return Err(bad("Este módulo es incompatible con uno instalado.")); }
    }
    let n=sqlx::query("UPDATE module_stock SET quantity=quantity-1 WHERE user_id=$1 AND module_id=$2 AND tier=$3 AND quantity>=1").bind(ctx.id).bind(input.module_id).bind(input.tier).execute(&mut *tx).await?.rows_affected();
    if n==0 { return Err(bad("No tienes una copia libre de ese módulo y grado.")); }
    let mut deltas=serde_json::to_value(&config.deltas).unwrap();
    for v in deltas.as_object_mut().unwrap().values_mut() { *v=json!(scale(v.as_i64().unwrap_or(0) as i32,input.tier as usize)); }
    let snapshot=json!({"code":code,"name":row.get::<String,_>(1),"deltas":deltas,"grants":config.grants,"excludes":config.excludes});
    sqlx::query("INSERT INTO program_modules VALUES($1,$2,$3,$4,$5,$6)").bind(input.software_id).bind(input.slot).bind(input.module_id).bind(input.tier).bind(ctx.id).bind(snapshot).execute(&mut *tx).await?;
    sync_size(&mut tx,input.software_id,server).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}
async fn unequip(state:web::Data<AppState>,ctx:Ctx,input:web::Json<Remove>)->Result<HttpResponse,ApiError> {
    let mut tx=state.pool.begin().await?;
    let (_,_,server)=lock_program(&mut tx,ctx.id,input.software_id).await?;
    let n=sqlx::query("DELETE FROM program_modules WHERE software_id=$1 AND slot=$2 AND owner_id=$3").bind(input.software_id).bind(input.slot).bind(ctx.id).execute(&mut *tx).await?.rows_affected();
    if n==0 { return Err(ApiError::NotFound); }
    sync_size(&mut tx,input.software_id,server).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}
async fn catalog(state:web::Data<AppState>,_:Ctx)->Result<HttpResponse,ApiError> {
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'code',code,'name',name,'config',workshop_config) FROM research_modules ORDER BY id").fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows))
}
async fn save_template(state:web::Data<AppState>,_:Ctx,input:web::Json<Template>)->Result<HttpResponse,ApiError> {
    let c=&input.config;
    if input.code.is_empty() || input.code.len()>48 || !input.code.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_') || input.name.trim().is_empty() || input.name.chars().count()>60 { return Err(bad("Código: minúsculas, números y guiones bajos (máx. 48). Nombre: 1–60 caracteres.")); }
    if !(1..=9999).contains(&c.min_level)|| !(1..=6).contains(&c.max_copies)||c.allowed_slots.is_empty()||c.allowed_slots.iter().any(|s| !(1..=6).contains(s))||c.excludes.len()>64 { return Err(bad("Revisa las restricciones de nivel, copias y ranuras (1–6).")); }
    let mut caps:Vec<String>=sqlx::query_scalar("SELECT code FROM software_capabilities").fetch_all(&state.pool).await?;
    if c.allowed_capabilities.iter().any(|s|!caps.contains(s)) { return Err(bad("Capacidad de destino desconocida.")); }
    caps.extend(["antivirus_scan".into(),"antivirus_clean".into()]);
    if c.grants.iter().any(|s|!caps.contains(s)) { return Err(bad("Función concedida desconocida.")); }
    let deltas=serde_json::to_value(&c.deltas).unwrap();
    for (k,v) in deltas.as_object().unwrap() {
        let limit=if ["cpu","ram","net","weight"].contains(&k.as_str()) {10000} else {100};
        if v.as_i64().unwrap_or(0).abs()>limit { return Err(bad("Atributos: −100 a 100; costes: −10000 a 10000.")); }
    }
    let conflict:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM research_modules WHERE code=$1 AND ($2::bigint IS NULL OR id<>$2))").bind(&input.code).bind(input.id).fetch_one(&state.pool).await?;
    if conflict { return Err(bad("Ya existe un módulo con ese código.")); }
    let value=serde_json::to_value(c).unwrap();
    let id:i64=if let Some(id)=input.id {
        sqlx::query_scalar("UPDATE research_modules SET name=$2,workshop_config=$3 WHERE id=$1 AND code=$4 RETURNING id").bind(id).bind(input.name.trim()).bind(value).bind(&input.code).fetch_optional(&state.pool).await?.ok_or_else(||bad("No existe el módulo o intentaste cambiar su código estable."))?
    } else {
        sqlx::query_scalar("INSERT INTO research_modules(code,name,workshop_config) VALUES($1,$2,$3) RETURNING id").bind(&input.code).bind(input.name.trim()).bind(value).fetch_one(&state.pool).await?
    };
    Ok(HttpResponse::Ok().json(json!({"id":id})))
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn progressive_slots(){assert_eq!([0,1,2,3,5,6,10,15,21,99].map(slots),[1,1,1,2,2,3,4,5,6,6]);}
    #[test] fn graded_stats(){assert_eq!((0..5).map(|t|scale(30,t)).collect::<Vec<_>>(),vec![30,41,54,72,96]);assert_eq!(scale(-10,4),-32);}
}
