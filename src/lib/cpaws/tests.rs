use cpaws::*;
use machine::Machine;
use script;
use script::Script;
use object::execution;

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

#[test]
fn build_script_symbols() {
  let mut machine = Machine::new();
  let     nodes   = ~[Symbol(~"hello"), Symbol(~"world")];

  let Script(script_nodes) = build_script(&mut machine, nodes);

  assert!(script_nodes.len() == 2);

  match &script_nodes[0] {
    &script::ObjectNode(ref object_ref) =>
      assert!(object_ref == &machine.symbol("hello")),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match &script_nodes[1] {
    &script::ObjectNode(ref object_ref) =>
      assert!(object_ref == &machine.symbol("world")),

    _ => fail!("Expected second node to be an ObjectNode")
  }
}

#[test]
fn build_script_expressions() {
  let mut machine = Machine::new();
  let     nodes   = ~[Symbol(~"a"), Expression(~[Symbol(~"b"), Symbol(~"c")])];

  let Script(script_nodes) = build_script(&mut machine, nodes);

  assert!(script_nodes.len() == 2);

  match &script_nodes[0] {
    &script::ObjectNode(_) => (),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match &script_nodes[1] {
    &script::ExpressionNode(ref subexp_nodes) => {
      assert!(subexp_nodes.len() == 2);

      match &subexp_nodes[0] {
        &script::ObjectNode(_) => (),

        _ => fail!("Expected subexpression's first node to be an ObjectNode")
      }

      match &subexp_nodes[1] {
        &script::ObjectNode(_) => (),

        _ => fail!("Expected subexpression's second node to be an ObjectNode")
      }
    },

    _ => fail!("Expected second node to be an ExpressionNode")
  }
}

#[test]
fn build_script_executions() {
  let mut machine = Machine::new();
  let     nodes   = ~[Execution(~[Symbol(~"a")])];

  let Script(script_nodes) = build_script(&mut machine, nodes);

  assert!(script_nodes.len() == 1);

  match &script_nodes[0] {
    &script::ObjectNode(ref object_ref) =>
      match object_ref.lock().try_cast::<execution::Execution>() {
        Ok(execution) => {
          let &Script(ref sub_script_nodes) = execution.deref().root();

          assert!(sub_script_nodes.len() == 1)

          match &sub_script_nodes[0] {
            &script::ObjectNode(_) => (),

            _ => fail!("Expected execution's first node to be an ObjectNode")
          }
        },

        Err(_) => fail!("Expected first node to point at an Execution")
      },

    _ => fail!("Expected first node to be an ObjectNode")
  }
}
