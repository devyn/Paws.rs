use cpaws::*;
use machine::Machine;
use script;
use script::Script;
use object::execution;

fn test_parse_nodes(test_case: &str,
                    expected_result: Result<Vec<Node>, String>) {
  let result = parse_nodes(test_case, "<test_case>");

  if expected_result != result {
    fail!("expected {}, got {}", expected_result, result);
  }
}

#[test]
fn parse_nodes_bare_symbol() {
  test_parse_nodes(
    "hello",
    Ok(vec![
      Symbol("hello".to_string())])
  )
}

#[test]
fn parse_nodes_quoted_symbol() {
  test_parse_nodes(
    "\"hello\n world\"",
    Ok(vec![
      Symbol("hello\n world".to_string())])
  )
}

#[test]
fn parse_nodes_unicode_quotes() {
  test_parse_nodes(
    "“hello\n world”",
    Ok(vec![
      Symbol("hello\n world".to_string())])
  )
}

#[test]
fn parse_nodes_expression() {
  test_parse_nodes(
    "a (b c) d",
    Ok(vec![
      Symbol("a".to_string()),
      Expression(vec![
        Symbol("b".to_string()),
        Symbol("c".to_string())]),
      Symbol("d".to_string())])
  )
}

#[test]
fn parse_nodes_execution() {
  test_parse_nodes(
    "a {b c} d",
    Ok(vec![
      Symbol("a".to_string()),
      Execution(vec![
        Symbol("b".to_string()),
        Symbol("c".to_string())]),
      Symbol("d".to_string())])
  )
}

#[test]
fn parse_nodes_missing_terminators() {
  test_parse_nodes(
    "\"",
    Err("<test_case>:1:1: expected '\"' before end-of-input".to_string())
  );
  test_parse_nodes(
    "“",
    Err("<test_case>:1:1: expected '”' before end-of-input".to_string())
  );
  test_parse_nodes(
    "(",
    Err("<test_case>:1:1: expected ')' before end-of-input".to_string())
  );
  test_parse_nodes(
    "{",
    Err("<test_case>:1:1: expected '}' before end-of-input".to_string())
  );
}

#[test]
fn parse_nodes_unexpected_terminators() {
  test_parse_nodes(
    "”",
    Err("<test_case>:1:1: unexpected terminator '”'".to_string())
  );
  test_parse_nodes(
    ")",
    Err("<test_case>:1:1: unexpected terminator ')'".to_string())
  );
  test_parse_nodes(
    "}",
    Err("<test_case>:1:1: unexpected terminator '}'".to_string())
  );
}

#[test]
fn build_script_symbols() {
  let machine = Machine::new();
  let nodes   = [Symbol("hello".to_string()),
                 Symbol("world".to_string())];

  let Script(script_nodes) = build_script(&machine, nodes);

  assert!(script_nodes.len() == 2);

  match script_nodes.get(0) {
    &script::ObjectNode(ref object_ref) =>
      assert!(object_ref.eq_as_symbol(&machine.symbol("hello"))),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match script_nodes.get(1) {
    &script::ObjectNode(ref object_ref) =>
      assert!(object_ref.eq_as_symbol(&machine.symbol("world"))),

    _ => fail!("Expected second node to be an ObjectNode")
  }
}

#[test]
fn build_script_expressions() {
  let machine = Machine::new();
  let nodes   = [Symbol("a".to_string()),
                 Expression(vec![Symbol("b".to_string()),
                                 Symbol("c".to_string())])];

  let Script(script_nodes) = build_script(&machine, nodes);

  assert!(script_nodes.len() == 2);

  match script_nodes.get(0) {
    &script::ObjectNode(_) => (),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match script_nodes.get(1) {
    &script::ExpressionNode(ref subexp_nodes) => {
      assert!(subexp_nodes.len() == 2);

      match subexp_nodes.get(0) {
        &script::ObjectNode(_) => (),

        _ => fail!("Expected subexpression's first node to be an ObjectNode")
      }

      match subexp_nodes.get(1) {
        &script::ObjectNode(_) => (),

        _ => fail!("Expected subexpression's second node to be an ObjectNode")
      }
    },

    _ => fail!("Expected second node to be an ExpressionNode")
  }
}

#[test]
fn build_script_executions() {
  let machine = Machine::new();
  let nodes   = [Execution(vec![Symbol("a".to_string())])];

  let Script(script_nodes) = build_script(&machine, nodes);

  assert!(script_nodes.len() == 1);

  match script_nodes.get(0) {
    &script::ObjectNode(ref object_ref) =>
      match object_ref.lock().try_cast::<execution::Execution>() {
        Ok(execution) => {
          let &Script(ref sub_script_nodes) = execution.deref().root();

          assert!(sub_script_nodes.len() == 1)

          match sub_script_nodes.get(0) {
            &script::ObjectNode(_) => (),

            _ => fail!("Expected execution's first node to be an ObjectNode")
          }
        },

        Err(_) => fail!("Expected first node to point at an Execution")
      },

    _ => fail!("Expected first node to be an ObjectNode")
  }
}
