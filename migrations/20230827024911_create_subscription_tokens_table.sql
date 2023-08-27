create table subscription_tokens (
    subscription_token text not null primary key,
    subscriber_id uuid not null references subscriptions(id)
);
