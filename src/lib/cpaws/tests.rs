use cpaws::*;
use machine::Machine;
use script::*;
use object::ObjectRef;
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
    "a[b c] d",
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
    "a{b c} d",
    Ok(vec![
      Symbol("a".to_string()),
      Execution(vec![
        Symbol("b".to_string()),
        Symbol("c".to_string())]),
      Symbol("d".to_string())])
  )
}

#[test]
fn parse_nodes_semicolon() {
  test_parse_nodes(
    ";a;\nb",
    Ok(vec![
      Semicolon,
      Symbol("a".to_string()),
      Semicolon,
      Symbol("b".to_string())])
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
    "[",
    Err("<test_case>:1:1: expected ']' before end-of-input".to_string())
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
    "]",
    Err("<test_case>:1:1: unexpected terminator ']'".to_string())
  );
  test_parse_nodes(
    "}",
    Err("<test_case>:1:1: unexpected terminator '}'".to_string())
  );
}

enum InstructionExpectation<'a> {
  ExpectInstruction(Instruction),
  ExpectPushSymbol(&'a str),
  ExpectPush(|&ObjectRef|:'a)
}

fn expect_instructions<'a>(
                       instructions: &'a [Instruction],
                       expectations: Vec<InstructionExpectation<'a>>) {

  assert!(instructions.len() == expectations.len(),
    "expected {} instructions, got {}",
    expectations.len(), instructions.len());

  for (instruction, expectation)
      in instructions.iter().zip(expectations.move_iter()) {

    match expectation {
      ExpectInstruction(ref i) =>
        if i != instruction {
          fail!("expected {}, got {}", i, instruction);
        },
      ExpectPushSymbol(s) =>
        match *instruction {
          Push(ref object)
            if object.symbol_ref().expect("not a Symbol")
                 .as_slice() == s => (),

          _ =>
            fail!("expected Push symbol \"{}\", got {}", s, instruction)
        },
      ExpectPush(block) =>
        match *instruction {
          Push(ref object) =>
            block(object),

          _ =>
            fail!("expected Push(..), got {}", instruction)
        }
    }
  }
}

#[test]
fn build_script_symbols() {
  let machine = Machine::new();
  let nodes   = [Symbol("hello".to_string()),
                 Symbol("world".to_string())];

  let Script(instructions) = build_script(&machine, nodes);

  expect_instructions(
    instructions.as_slice(),
    vec![
      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),
      ExpectPushSymbol("hello"),
      ExpectInstruction(Combine),
      ExpectPushSymbol("world"),
      ExpectInstruction(Combine)
    ]);
}

#[test]
fn build_script_expressions() {
  let machine = Machine::new();
  let nodes   = [Symbol("a".to_string()),
                 Expression(vec![Symbol("b".to_string()),
                                 Symbol("c".to_string())])];

  let Script(instructions) = build_script(&machine, nodes);

  expect_instructions(
    instructions.as_slice(),
    vec![
      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),
      ExpectPushSymbol("a"),

      ExpectInstruction(Combine),
      ExpectInstruction(PushLocals),
      ExpectPushSymbol("b"),
      ExpectInstruction(Combine),

      ExpectPushSymbol("c"),
      ExpectInstruction(Combine),
      ExpectInstruction(Combine)
    ]);
}

#[test]
fn build_script_executions() {
  let machine = Machine::new();
  let nodes   = [Execution(vec![Symbol("a".to_string())])];

  let Script(instructions) = build_script(&machine, nodes);

  expect_instructions(
    instructions.as_slice(),
    vec![
      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),
      ExpectPush(|o| {
        let execution =
          o.lock().try_cast::<execution::Execution>()
            .ok().expect("expected Execution");

        let Script(ref instructions) = *execution.deref().root();

        expect_instructions(
          instructions.as_slice(),
          vec![
            ExpectInstruction(Discard),
            ExpectInstruction(PushLocals),
            ExpectPushSymbol("a"),
            ExpectInstruction(Combine)
          ]);
      }),
      ExpectInstruction(Combine)
    ]);
}

#[test]
fn build_script_semicolons() {
  let machine = Machine::new();
  let nodes   = [Semicolon,
                 Symbol("a".to_string()),
                 Semicolon,
                 Symbol("b".to_string()),
                 Symbol("c".to_string())];

  let Script(instructions) = build_script(&machine, nodes);

  expect_instructions(
    instructions.as_slice(),
    vec![
      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),

      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),

      ExpectPushSymbol("a"),
      ExpectInstruction(Combine),

      ExpectInstruction(Discard),
      ExpectInstruction(PushLocals),

      ExpectPushSymbol("b"),
      ExpectInstruction(Combine),

      ExpectPushSymbol("c"),
      ExpectInstruction(Combine)
    ]);
}
