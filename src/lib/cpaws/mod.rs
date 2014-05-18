//! Implements parsing of cPaws ('canonical paws').

use std::str::Chars;
use std::char::is_whitespace;

use script;
use script::Script;
use machine::Machine;
use object::symbol;

use object::Object;
use std::rc::Rc;

#[cfg(test)]
mod tests;

/// Represents a cPaws node, which is either a symbol string, an expression of
/// subnodes, or an execution of subnodes.
#[deriving(Eq)]
pub enum Node {
  Symbol(~str),
  Expression(~[Node]),
  Execution(~[Node])
}

/// Holds the state of the parser, including character iterator and position.
struct ParserState<'r> {
  chars:    &'r mut Chars<'r>,
  filename: &'r str,
  line:     int,
  column:   int
}

impl<'r> ParserState<'r> {
  /// Formats an error string based on the current parser state and puts it in
  /// a `Result` as an `Err`
  fn error<T>(&self, message: &str) -> Result<T, ~str> {
    Err(format!("{}:{}:{}: {}", self.filename, self.line, self.column, message))
  }
}

/// Parses a string into a vector of nodes representing the root of the script.
///
/// # Returns
///
/// `Err(message)` if parsing failed; `Ok(nodes)` otherwise.
pub fn parse_nodes(text: &str, filename: &str) -> Result<~[Node], ~str> {
  let mut chars = text.chars();

  let mut state = ParserState {
    chars:    &mut chars,
    filename: filename,
    line:     1,
    column:   1
  };

  parse_nodes_until(&mut state, None)
}

/// Parses nodes until, if a terminator is given, the terminator appears, or if
/// no terminator is given, the end of the `chars` iterator is reached.
///
/// # Returns
///
/// `Err(message)` if parsing failed; `Ok(nodes)` otherwise.
fn parse_nodes_until(state: &mut ParserState, terminator: Option<char>) ->
   Result<~[Node], ~str> {

  let mut nodes = ~[];

  loop {
    match state.chars.next() {
      // If we're at the end of the input
      None =>
        match terminator {
          // If we had a terminator we were expecting first, throw an error
          Some(c) =>
            return state.error(format!("expected '{}' before end-of-input", c)),

          // Else cleanly return
          None => break
        },

      // Skip newlines, but also increment line and set column to zero for error
      // messages
      Some('\n') => {
        state.line += 1;
        state.column = 1;
        continue;
      },

      // If we've specified a terminator and we get it, immediately cleanly
      // return
      Some(c) if Some(c) == terminator => break,

      // Skip whitespace
      Some(c) if is_whitespace(c) => (),

      // (expression)
      Some('(') =>
        nodes.push(Expression(
          try!(parse_nodes_until(state, Some(')'))))),

      // {expression}
      Some('{') =>
        nodes.push(Execution(
          try!(parse_nodes_until(state, Some('}'))))),

      // "string"
      Some('"') =>
        nodes.push(Symbol(
          try!(parse_string_until(state, '"')))),

      // “string”
      Some('“') =>
        nodes.push(Symbol(
          try!(parse_string_until(state, '”')))),

      // If we get any terminators that we *weren't* expecting, those are
      // errors.
      Some(c @ ')') | Some(c @ '}') | Some(c @ '”') =>
        return state.error(format!("unexpected terminator '{}'", c)),

      // Any other character is the start of a bare symbol
      Some(c) =>
        nodes.push(Symbol(
          parse_bare_symbol(state, c)))
    }

    state.column += 1;
  }

  Ok(nodes)
}

/// Parses into a string until the terminator appears.
///
/// # Returns
///
/// `Err(message)` if end-of-input was reached before the terminator was found;
/// `Ok(string)` otherwise.
fn parse_string_until(state: &mut ParserState, terminator: char) ->
   Result<~str, ~str> {

  let mut string = ~"";

  loop {
    match state.chars.next() {
      // End-of-input is always an error here, since the terminator is required
      None =>
        return state.error(format!(
          "expected '{}' before end-of-input", terminator)),

      // Return cleanly if we get our terminator
      Some(c) if c == terminator => break,

      // Increment line/column information if we get a newline, and also
      // include it as part of our string
      Some('\n') => {
        state.line += 1;
        state.column = 1;
        string.push_char('\n');
        continue;
      },

      // Any other char is part of our string. cPaws doesn't have any escape
      // sequences to deal with; specifically any Unicode codepoint that *isn't*
      // the terminator of the string will be included.
      Some(c) => string.push_char(c)
    }

    state.column += 1;
  }

  Ok(string)
}

/// Like 'parse_string_until`, but specialized for the rules of a bare symbol
/// without quotes.
///
/// Unlike 'parse_string_until', it never returns an error message.
fn parse_bare_symbol(state: &mut ParserState, first_char: char) -> ~str {

  let mut string = ~"";

  // There isn't really a way to push the first char back onto `state.chars`
  // from `parse_nodes_until` so we have to handle it specially
  string.push_char(first_char);

  loop {
    match state.chars.peekable().peek() {
      None => break,

      Some(c) =>
        match *c {
          // A bare symbol is ended by any special characters or whitespace
          '{' | '}' | '(' | ')'  => break,
          '"' | '“' | '”'        => break,
          _ if is_whitespace(*c) => break,

          // Anything else gets to be part of the string
          _ => {
            string.push_char(*c);
            state.chars.next();
            state.column += 1;
          }
        }
    }
  }

  string
}

/// Converts a vector of cPaws nodes into a Paws Script.
pub fn build_script(machine: &mut Machine, nodes: &[Node]) -> Script {
  Script(
    nodes.iter().map(|node|
      cpaws_node_to_script_node(machine, node)
    ).collect()
  )
}

/// Converts `cpaws::Node` -> `script::Node` for a given Machine.
fn cpaws_node_to_script_node(machine: &mut Machine, node: &Node)
   -> script::Node {
  match node {
    &Symbol(ref string) => {
      let object: ~Object =
        ~symbol::Symbol::new(string.as_slice(), &mut machine.symbol_map);

      script::ObjectNode(Rc::new(object))
    },

    &Expression(ref nodes) =>
      script::ExpressionNode(
        nodes.iter().map(|node|
          cpaws_node_to_script_node(machine, node)
        ).collect()
      ),

    &Execution(ref nodes) =>
      unimplemented!()
  }
}
