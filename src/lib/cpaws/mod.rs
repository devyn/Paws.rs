//! Implements parsing of cPaws ('canonical paws').

use std::str::Chars;
use std::char::is_whitespace;

use script::*;
use machine::Machine;

#[cfg(test)]
mod tests;

/// Represents a cPaws node, which is either a symbol string, an expression of
/// subnodes, or an execution of subnodes.
#[deriving(Clone, Eq, PartialEq, Show)]
pub enum Node {
  /// A symbol string. The cPaws representation of a Symbol object.
  Symbol(String),

  /// An expression of subnodes. The semantics of this are such that everything
  /// within this is run, and then the result is combined to the left.
  Expression(Vec<Node>),

  /// The cPaws representation of an Execution object.
  Execution(Vec<Node>)
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
  fn error<T>(&self, message: String) -> Result<T, String> {
    Err(format!("{}:{}:{}: {}", self.filename, self.line, self.column, message))
  }
}

/// Parses a string into a vector of nodes representing the root of the script.
///
/// # Returns
///
/// `Err(message)` if parsing failed; `Ok(nodes)` otherwise.
pub fn parse_nodes(text: &str, filename: &str) -> Result<Vec<Node>, String> {
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
fn parse_nodes_until(state: &mut ParserState, terminator: Option<char>)
                     -> Result<Vec<Node>, String> {

  let mut nodes = Vec::new();

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

      // [expression]
      Some('[') =>
        nodes.push(Expression(
          try!(parse_nodes_until(state, Some(']'))))),

      // {execution}
      Some('{') =>
        nodes.push(Execution(
          try!(parse_nodes_until(state, Some('}'))))),

      // "symbol"
      Some('"') =>
        nodes.push(Symbol(
          try!(parse_string_until(state, '"')))),

      // “symbol”
      Some('“') =>
        nodes.push(Symbol(
          try!(parse_string_until(state, '”')))),

      // If we get any terminators that we *weren't* expecting, those are
      // errors.
      Some(c @ ']') | Some(c @ '}') | Some(c @ '”') =>
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
fn parse_string_until(state: &mut ParserState, terminator: char)
                      -> Result<String, String> {

  let mut string = String::new();

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
fn parse_bare_symbol(state: &mut ParserState, first_char: char) -> String {

  let mut string = String::new();

  // There isn't really a way to push the first char back onto `state.chars`
  // from `parse_nodes_until` so we have to handle it specially
  string.push_char(first_char);

  loop {
    match state.chars.peekable().peek() {
      None => break,

      Some(c) =>
        match *c {
          // A bare symbol is ended by any special characters or whitespace
          '{' | '}' | '[' | ']'  => break,
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

/// Converts a slice of cPaws nodes into a Paws Script.
pub fn build_script(machine: &Machine, nodes: &[Node]) -> Script {
  Script({
    let mut instructions = vec![Discard, PushLocals]; // pristine

    for node in nodes.iter() {
      compile(machine, &mut instructions, node);
    }

    debug!("build_script instructions: {}", instructions);

    instructions
  })
}

/// Compiles a `Node` into instructions and places them on a vector.
fn compile(machine:      &Machine,
           instructions: &mut Vec<Instruction>,
           node:         &Node) {
  match node {
    &Symbol(ref string) => {
      instructions.push(Push(machine.symbol(string.as_slice())));
      instructions.push(Combine);
    },

    &Expression(ref nodes) => {
      if nodes.is_empty() {
        // Empty expression special case = "self"
        instructions.push(PushSelf);
        instructions.push(Combine);
      } else {
        instructions.push(PushLocals);

        for node in nodes.iter() {
          compile(machine, instructions, node);
        }

        instructions.push(Combine);
      }
    },

    &Execution(ref nodes) => {
      let execution = machine.execution(
        build_script(machine, nodes.as_slice()));

      instructions.push(Push(execution));
      instructions.push(Combine);
    }
  }
}
