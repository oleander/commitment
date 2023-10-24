use crate::commitment::*;

#[test]
fn test_to_commit() {
  assert_eq!(
    commit("INVALID INVALID", "ABC-123 Message"),
    Ok("ABC-123 Message".to_string())
  );
  assert_eq!(
    commit("INVALID", "ABC-123 Message"),
    Ok("ABC-123 Message".to_string())
  );
  assert_eq!(
    commit("", "ABC-123 Message"),
    Ok("ABC-123 Message".to_string())
  );
  assert_eq!(
    commit("ABC-123", "Message"),
    Ok("ABC-123 Message".to_string())
  );
  assert_eq!(
    commit("ABC-123-DEF", "Tail"),
    Ok("ABC-123 Tail".to_string())
  );
  assert_eq!(commit("", "message"), Ok("Message".to_string()));
  assert_eq!(commit("X", "message"), Ok("Message".to_string()));
  assert_eq!(
    commit("ABC-123", "message"),
    Ok("ABC-123 Message".to_string())
  );
  assert!(commit("ABC-123", "ABC-123 Tail1 Tail2").is_ok());
  assert!(commit("ABC-123", "DEF-456 Tail").is_err());
  assert!(commit("ABC-123", "ABC-123 Tail").is_ok());
  assert!(commit("ABC-123", "DEF-456").is_err());
  assert!(commit("ABC-123-DEF", "").is_err());
  assert!(commit("ABC-123", "").is_err());
  assert!(commit("", "").is_err());
}

#[test]
fn test_capitalize_first() {
  assert_eq!(capitalize_first("abc"), "Abc");
  assert_eq!(capitalize_first("Abc"), "Abc");
  assert_eq!(capitalize_first(""), "");
}


#[test]
fn test_to_ticket() {
  assert_eq!(
    "Head Tail1 Tail2 Tail3".to_ticket(),
    (None, Some("Head Tail1 Tail2 Tail3"))
  );
  assert_eq!(
    "ABC-123 Tail1 Tail2".to_ticket(),
    (Some("ABC-123"), Some("Tail1 Tail2"))
  );
  assert_eq!("ABC-123 Tail".to_ticket(), (Some("ABC-123"), Some("Tail")));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("Head".to_ticket(), (None, Some("Head")));
  assert_eq!("".to_ticket(), (None, None));
}
