//! Quien puede usar cada orden de un sitio.
//!
//! El motor no sabe que es un banco. Sabe que hay dos familias de ordenes -las normales, que
//! usa cualquiera, y las integradas, que llevan la interfaz dentro del HTML del sitio- y que
//! cada concesion dice a quien alcanza. Que hoy solo la banca este concedida, y solo a los
//! bancos, es una fila de la tabla y no una regla del codigo.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::{auth::Ctx, error::ApiError};

use super::webscript::catalog::{self, Family};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GrantRow {
    pub id: i64,
    pub action: String,
    pub family: String,
    pub scope: String,
    pub organization_id: Option<i64>,
    pub npc_id: Option<i64>,
    /// El nombre de a quien alcanza, para no tener que resolverlo en el cliente.
    pub target_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GrantInput {
    pub action: String,
    pub family: String,
    pub scope: String,
    #[serde(default)]
    pub organization_id: Option<i64>,
    #[serde(default)]
    pub npc_id: Option<i64>,
}

/// Una orden, tal como la ve el panel: como se escribe, de que familia es y para que sirve.
#[derive(Debug, Serialize)]
pub struct ActionInfo {
    pub verb: &'static str,
    pub family: &'static str,
    pub help: &'static str,
}

pub fn catalog_actions() -> Vec<ActionInfo> {
    catalog::CATALOG
        .iter()
        .map(|spec| ActionInfo {
            verb: spec.verb,
            family: spec.family.as_str(),
            help: spec.help,
        })
        .collect()
}

pub async fn list(pool: &PgPool, ctx: Ctx) -> Result<Vec<GrantRow>, ApiError> {
    ctx.require_admin()?;

    Ok(sqlx::query_as::<_, GrantRow>(
        r#"
        SELECT g.id, g.action, g.family, g.scope, g.organization_id, g.npc_id,
               COALESCE(o.name, i.name) AS target_name
        FROM action_grants g
        LEFT JOIN organizations o ON o.id = g.organization_id
        LEFT JOIN npc_info i ON i.npc_id = g.npc_id
        ORDER BY g.family, g.scope, g.id
        "#,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create(pool: &PgPool, ctx: Ctx, input: GrantInput) -> Result<i64, ApiError> {
    ctx.require_admin()?;

    let family = match input.family.as_str() {
        "normal" => Family::Normal,
        "integrated" => Family::Integrated,
        other => {
            return Err(ApiError::BadRequest(format!(
                "GRANT_FAMILY: `{other}` no es una familia. Hay `normal` e `integrated`"
            )))
        }
    };

    // Un `*` abre la familia entera; cualquier otra cosa tiene que ser una orden de esa
    // familia. Conceder `bank_login` como normal seria una concesion que no aplica a nada.
    if input.action != "*"
        && !catalog::find(&input.action).is_some_and(|spec| spec.family == family)
    {
        return Err(ApiError::BadRequest(format!(
            "GRANT_ACTION: `{}` no es una orden de la familia {}",
            input.action, input.family
        )));
    }

    sqlx::query_scalar(
        r#"
        INSERT INTO action_grants(action, family, scope, organization_id, npc_id)
        VALUES($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(&input.action)
    .bind(&input.family)
    .bind(&input.scope)
    .bind(input.organization_id)
    .bind(input.npc_id)
    .fetch_one(pool)
    .await
    .map_err(|error| match &error {
        // El CHECK de la tabla es quien sabe que alcance necesita que destino.
        sqlx::Error::Database(_) => ApiError::BadRequest(
            "GRANT_TARGET: ese alcance necesita su destino, y solo el suyo".into(),
        ),
        _ => ApiError::from(error),
    })
}

pub async fn remove(pool: &PgPool, ctx: Ctx, id: i64) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let done = sqlx::query("DELETE FROM action_grants WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if done.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}
