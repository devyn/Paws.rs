use cpaws::*;
use machine::Machine;
use script;
use script::Script;
use object::Object;
use object::symbol;

use std::any::Any;
use std::intrinsics::TypeId;
use std::io::MemWriter;

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

/// A really, really ugly way to test the symbol within an Object reference.
///
/// FIXME: Need something better, especially since we're going to want to
/// compare symbols in the future.
fn test_symbol_in_object(object: &~Object, string: &str, machine: &Machine) {

  // First make sure the types match up.
  if object.get_type_id() == TypeId::of::<symbol::Symbol>() {

    // Next, format both the test target and the case and compare the result of
    // the formatting.
    let mut test_writer = MemWriter::new();
    let mut case_writer = MemWriter::new();

    object.fmt_paws(&mut test_writer, machine).unwrap();

    (write!(&mut case_writer, "Symbol[{}]", string)).unwrap();

    assert!(test_writer.unwrap() == case_writer.unwrap())
  } else {
    fail!("Object is not a Symbol")
  }
}

#[test]
fn build_script_symbols() {
  let mut machine = Machine::new();
  let     nodes   = ~[Symbol(~"hello"), Symbol(~"world")];

  let Script(script_nodes) = build_script(&mut machine, nodes);

  if script_nodes.len() != 2 {
    fail!("Expected a script with 2 nodes, got {}", script_nodes.len())
  }

  match &script_nodes[0] {
    &script::ObjectNode(ref object) =>
      test_symbol_in_object(object, "hello", &machine),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match &script_nodes[1] {
    &script::ObjectNode(ref object) =>
      test_symbol_in_object(object, "world", &machine),

    _ => fail!("Expected second node to be an ObjectNode")
  }
}

#[test]
fn build_script_expressions() {
  let mut machine = Machine::new();
  let     nodes   = ~[Symbol(~"a"), Expression(~[Symbol(~"b"), Symbol(~"c")])];

  let Script(script_nodes) = build_script(&mut machine, nodes);

  if script_nodes.len() != 2 {
    fail!("Expected a script with 2 nodes, got {}", script_nodes.len());
  }

  match &script_nodes[0] {
    &script::ObjectNode(_) => (),

    _ => fail!("Expected first node to be an ObjectNode")
  }

  match &script_nodes[1] {
    &script::ExpressionNode(ref subexp_nodes) => {
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
