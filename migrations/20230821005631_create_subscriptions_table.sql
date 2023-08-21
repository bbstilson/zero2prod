create table subscriptions(
  id uuid primary key,
  email text not null unique,
  name text not null,
  subscribed_at timestamp without time zone default now() not null
);
