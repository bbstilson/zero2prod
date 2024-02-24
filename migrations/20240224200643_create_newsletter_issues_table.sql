create table newsletter_issues (
  newsletter_issue_id uuid primary key,
  title text not null,
  html_content text not null,
  text_content text not null,
  published_at timestamptz not null
);
