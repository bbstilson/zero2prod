pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, ()> {
        match garde::rules::email::parse_email(&s) {
            Ok(_) => Ok(SubscriberEmail(s)),
            Err(_) => Err(()),
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
