pub(super) fn is_email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    let local_is_valid = !local.is_empty()
        && local.len() <= 64
        && !local.chars().any(|c| c.is_whitespace() || c.is_control());
    let label_is_valid = |label: &str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_alphanumeric() || c == '-')
    };
    text.len() <= 254 && local_is_valid && domain.split('.').all(label_is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_need_one_at_sign_no_spaces_and_a_plain_domain() {
        let valid = [
            "sang@example.com",
            "a.b+tag@sub.example.co.th",
            "root@localhost",
        ];
        let invalid = [
            "plain",
            "@example.com",
            "sang@",
            "sang@exa mple.com",
            "a@b@c.com",
            "sang@-bad.com",
            "sang@example..com",
        ];

        for email in valid {
            assert!(is_email(email), "{email} was refused");
        }
        for email in invalid {
            assert!(!is_email(email), "{email} was allowed");
        }
    }
}
