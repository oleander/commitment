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
