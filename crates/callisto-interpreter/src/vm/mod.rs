use std::collections::HashMap;

use thiserror::Error;
use value::Value;

use crate::parser::syntax::Syntax;

pub mod value;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeError {
    #[error("Lexing error: {0}")]
    LexingError(#[from] crate::lexer::LexingError),

    #[error("Parsing error: {0}")]
    ParsingError(#[from] crate::parser::ParsingError),

    #[error("Invalid type: {0}")]
    TypeError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Undefined identifier: {0}")]
    UndefinedIdentifier(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Runtime error: {0}")]
    Other(String),
}

pub struct Runtime {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, Syntax>,
    pub call_stack: Vec<String>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn execute_str(&mut self, input: &str) -> Result<Value, RuntimeError> {
        let syntax_tree = crate::parser::parse_str(input)?;
        let mut result = Value::Null;
        for syntax in syntax_tree {
            result = self.execute(syntax)?;
        }
        Ok(result)
    }

    pub fn execute(&mut self, syntax: Syntax) -> Result<Value, RuntimeError> {
        match syntax {
            Syntax::Number(value) => Ok(Value::Number(value)),
            Syntax::Boolean(value) => Ok(Value::Boolean(value)),
            Syntax::Symbol(value) => Ok(Value::Symbol(value)),
            Syntax::String(value) => Ok(Value::String(value)),
            Syntax::List(elements) => {
                if elements.is_empty() {
                    return Ok(Value::Null);
                }
                let first = elements[0].clone();

                if let Syntax::Identifier(name) | Syntax::Operator(name) = first {
                    if let Some(value) = self.variables.get(&name) {
                        return Ok(value.clone());
                    }

                    match self.execute_builtin_function(&name, &elements[1..]) {
                        Ok(value) => return Ok(value),
                        Err(RuntimeError::UndefinedFunction(_)) => {}
                        Err(e) => return Err(e),
                    }

                    match self.execute_function(&name, &elements[1..]) {
                        Ok(value) => return Ok(value),
                        Err(RuntimeError::UndefinedFunction(_)) => {}
                        Err(e) => return Err(e),
                    }

                    Err(RuntimeError::UndefinedIdentifier(name))
                } else {
                    let mut values = Vec::new();
                    for element in elements {
                        let value = self.execute(element)?;
                        values.push(value);
                    }
                    Ok(Value::List(values))
                }
            }
            syntax => Err(RuntimeError::Other(format!("Invalid syntax: {:?}", syntax))),
        }
    }

    fn execute_function(
        &mut self,
        function: &str,
        arguments: &[Syntax],
    ) -> Result<Value, RuntimeError> {
        self.call_stack.push(function.to_string());
        self.call_stack.pop();
        Err(RuntimeError::Other(format!(
            "Function not found: {:?}",
            function
        )))
    }

    fn execute_builtin_function(
        &mut self,
        function: &str,
        arguments: &[Syntax],
    ) -> Result<Value, RuntimeError> {
        self.call_stack.push(function.to_string());
        let value = match function {
            "print" => {
                for arg in arguments {
                    let value = self.execute(arg.clone())?;
                    println!("{:?}", value);
                }
                Ok(Value::Null)
            }
            "+" => {
                let mut sum = 0.0;
                for arg in arguments {
                    let value = self.execute(arg.clone())?;
                    if let Value::Number(num) = value {
                        sum += num;
                    } else {
                        return Err(RuntimeError::TypeError(format!(
                            "Expected number, found: {:?}",
                            value
                        )));
                    }
                }
                Ok(Value::Number(sum))
            }
            "-" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::TypeError(format!(
                        "Expected 2 arguments for '-', found: {}",
                        arguments.len()
                    )));
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
                    Ok(Value::Number(a - b))
                } else {
                    Err(RuntimeError::TypeError(format!(
                        "Expected numbers, found: {:?} and {:?}",
                        a, b
                    )))
                }
            }
            "*" => {
                let mut product = 1.0;
                for arg in arguments {
                    let value = self.execute(arg.clone())?;
                    if let Value::Number(num) = value {
                        product *= num;
                    } else {
                        return Err(RuntimeError::TypeError(format!(
                            "Expected number, found: {:?}",
                            value
                        )));
                    }
                }
                Ok(Value::Number(product))
            }
            "/" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::TypeError(format!(
                        "Expected 2 arguments for '/', found: {}",
                        arguments.len()
                    )));
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    Ok(Value::Number(a / b))
                } else {
                    Err(RuntimeError::TypeError(format!(
                        "Expected numbers, found: {:?} and {:?}",
                        a, b
                    )))
                }
            }
            "==" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::TypeError(format!(
                        "Expected 2 arguments for 'eq?', found: {}",
                        arguments.len()
                    )));
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                Ok(Value::Boolean(a == b))
            }
            "!=" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::TypeError(format!(
                        "Expected 2 arguments for '!=', found: {}",
                        arguments.len()
                    )));
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                Ok(Value::Boolean(a != b))
            }

            function => Err(RuntimeError::Other(format!(
                "Unknown function: {}",
                function
            ))),
        };

        self.call_stack.pop();
        value
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Result<Value, RuntimeError> {
        self.variables
            .get(name)
            .cloned()
            .ok_or(RuntimeError::UndefinedVariable(name.to_string()))
    }

    pub fn set_function(&mut self, name: String, function: Syntax) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&self, name: &str) -> Result<Syntax, RuntimeError> {
        self.functions
            .get(name)
            .cloned()
            .ok_or(RuntimeError::UndefinedVariable(name.to_string()))
    }

    pub fn call_stack(&self) -> &[String] {
        &self.call_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let mut runtime = Runtime::new();
        let result = runtime.execute_str("(+ 1 2 3)").unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_subtraction() {
        let mut runtime = Runtime::new();
        let result = runtime.execute_str("(- 5 3)").unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_multiplication() {
        let mut runtime = Runtime::new();
        let result = runtime.execute_str("(* 2 3 4)").unwrap();
        assert_eq!(result, Value::Number(24.0));
    }

    #[test]
    fn test_division() {
        let mut runtime = Runtime::new();
        let result = runtime.execute_str("(/ 8 2)").unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_nested() {
        let mut runtime = Runtime::new();
        let result = runtime.execute_str("(+ 1 (* 2 3))").unwrap();
        assert_eq!(result, Value::Number(7.0));
    }
}
