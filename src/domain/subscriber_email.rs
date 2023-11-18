pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        match garde::rules::email::parse_email(&s) {
            Ok(_) => Ok(SubscriberEmail(s)),
            Err(_) => Err(format!("{} is not a valid subscriber email.", s)),
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
