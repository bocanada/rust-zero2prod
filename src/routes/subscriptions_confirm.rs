use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip_all,
    fields(subscription_token = %query.subscription_token)
)]
pub async fn confirm(query: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let Ok(id) = get_subscriber_id_from_token(&pool, &query.subscription_token).await else {
        return HttpResponse::InternalServerError().finish();
    };

    match id {
        // Non-existing token!
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all)]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip_all)]
/// Gets the subscriber id from a token.
/// If the subscriber was already confirmed, it returns `Ok(None)`.
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> sqlx::Result<Option<Uuid>> {
    let maybe_id = sqlx::query!(
        "SELECT subscriber_id
         FROM subscription_tokens
              INNER JOIN subscriptions s ON s.id = subscriber_id
         WHERE subscription_token = $1
               AND s.status <> 'confirmed'",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        e
    })?;
    Ok(maybe_id.map(|r| r.subscriber_id))
}
