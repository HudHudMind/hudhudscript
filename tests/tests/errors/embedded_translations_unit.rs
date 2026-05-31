#[cfg(test)]
mod tests {
    use hudhudscript_errors::{embedded_error_catalog, localized_error_entry, ErrorCode};

    #[test]
    fn turkish_translations_are_embedded() {
        let catalog = embedded_error_catalog("tr").expect("embedded tr catalog");
        let translated = catalog
            .get(ErrorCode::ApprovalInvalidTransition)
            .expect("translated E0001");
        assert_eq!(translated.title, "Onay durumu geçişi geçersiz");
    }

    #[test]
    fn broken_or_empty_translations_fall_back_to_english() {
        let localized = localized_error_entry(ErrorCode::ApprovalInvalidTransition, "pt-BR");
        assert_eq!(
            localized.title,
            ErrorCode::ApprovalInvalidTransition.title()
        );
    }
}
