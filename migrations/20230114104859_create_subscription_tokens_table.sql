-- Add migration script here
CREATE TABLE subscription_tokens(
    subscription_token TEXT PRIMARY KEY NOT NULL,
    subscriber_id uuid NOT NULL
        REFERENCES subscriptions(id)
);

