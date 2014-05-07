use cpaws::*;

fn test_parse_nodes(test_case: &str, expected_result: Result<~[Node], ~str>) {
  let result = parse_nodes(test_case, "<test_case>");

  if expected_result != result {
    fail!("expected {:?}, got {:?}", expected_result, result);
  }
}

#[test]
fn parse_nodes_bare_symbol() {
  test_parse_nodes(
    &"hello",
    Ok(~[Symbol(~"hello")])
  )
}

#[test]
fn parse_nodes_quoted_symbol() {
  test_parse_nodes(
    &"\"hello\n world\"",
    Ok(~[Symbol(~"hello\n world")])
  )
}

#[test]
fn parse_nodes_unicode_quotes() {
  test_parse_nodes(
    &"“hello\n world”",
    Ok(~[Symbol(~"hello\n world")])
  )
}

#[test]
fn parse_nodes_expression() {
  test_parse_nodes(
    &"a (b c) d",
    Ok(~[Symbol(~"a"), Expression(~[Symbol(~"b"), Symbol(~"c")]), Symbol(~"d")])
  )
}

#[test]
fn parse_nodes_execution() {
  test_parse_nodes(
    &"a {b c} d",
    Ok(~[Symbol(~"a"), Execution(~[Symbol(~"b"), Symbol(~"c")]), Symbol(~"d")])
  )
}

#[test]
fn parse_nodes_missing_terminators() {
  test_parse_nodes(
    &"\"",
    Err(~"<test_case>:1:1: expected '\"' before end-of-input")
  );
  test_parse_nodes(
    &"“",
    Err(~"<test_case>:1:1: expected '”' before end-of-input")
  );
  test_parse_nodes(
    &"(",
    Err(~"<test_case>:1:1: expected ')' before end-of-input")
  );
  test_parse_nodes(
    &"{",
    Err(~"<test_case>:1:1: expected '}' before end-of-input")
  );
}

#[test]
fn parse_nodes_unexpected_terminators() {
  test_parse_nodes(
    &"”",
    Err(~"<test_case>:1:1: unexpected terminator '”'")
  );
  test_parse_nodes(
    &")",
    Err(~"<test_case>:1:1: unexpected terminator ')'")
  );
  test_parse_nodes(
    &"}",
    Err(~"<test_case>:1:1: unexpected terminator '}'")
  );
}
