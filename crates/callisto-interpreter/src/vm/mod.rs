use std::collections::HashMap;

use thiserror::Error;
use value::{Value, ValueType};

use crate::{
    lexer::LexingError,
    parser::{
        ParsingError, parse_str,
        syntax::{Syntax, SyntaxType},
    },
};

pub mod builtins;
pub mod value;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeError {
    #[error("Lexing error: {0}")]
    LexingError(#[from] LexingError),

    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingError),

    #[error("Invalid argument count: expected {expected}, found {found}")]
    InvalidArgumentCount { expected: usize, found: usize },

    #[error("Syntax error: expected {expected:?}, found {found:?}")]
    SyntaxError {
        expected: SyntaxType,
        found: SyntaxType,
    },

    #[error("Syntax error: {0:?}")]
    InvalidSyntax(SyntaxType),

    #[error("Type error: expected {expected:?}, found {found:?}")]
    TypeError {
        expected: ValueType,
        found: ValueType,
    },

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Undefined identifier: {0}")]
    UndefinedIdentifier(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid operation: {operation} between {left:?} and {right:?}")]
    InvalidOperation {
        operation: String,
        left: ValueType,
        right: ValueType,
    },

    #[error("Cannot convert to graph node: {0:?}")]
    CannotConvertToNode(ValueType),

    #[error("scope error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Syntax,
}

pub fn execute_str(input: &str) -> Result<Value, RuntimeError> {
    Vm::default().execute_str(input)
}

#[derive(Clone, PartialEq)]
pub struct Vm {}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute_str(&self, input: &str) -> Result<Value, RuntimeError> {
        Scope::new(self).execute_str(input)
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope<'vm> {
    pub vm: &'vm Vm,
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, FunctionDef>,
    pub call_stack: Vec<String>,
}

impl<'vm> Scope<'vm> {
    pub fn new(vm: &'vm Vm) -> Self {
        Self {
            vm,
            variables: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn execute_str(&mut self, input: &str) -> Result<Value, RuntimeError> {
        let syntax_tree = parse_str(input)?;
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
            Syntax::Identifier(name) => {
                if let Some(value) = self.variables.get(&name) {
                    return Ok(value.clone());
                }
                Err(RuntimeError::UndefinedVariable(name))
            }
            Syntax::List(elements) => {
                if elements.is_empty() {
                    return Ok(Value::Null);
                }
                let first = elements[0].clone();

                if let Syntax::Identifier(name) | Syntax::Operator(name) = first {
                    if let Some(value) = self.variables.get(&name) {
                        return Ok(value.clone());
                    }

                    self.call_stack.push(name.clone());
                    let result = self.execute_builtin_function(&name, &elements[1..]);
                    self.call_stack.pop();
                    match result {
                        Ok(value) => return Ok(value),
                        Err(RuntimeError::UndefinedFunction(_)) => {}
                        Err(e) => return Err(e),
                    }

                    self.call_stack.push(name.clone());
                    let result = self.execute_function(&name, &elements[1..]);
                    self.call_stack.pop();
                    match result {
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
            syntax => Err(RuntimeError::InvalidSyntax(syntax.syntax_type())),
        }
    }

    fn execute_function(
        &mut self,
        function: &str,
        arguments: &[Syntax],
    ) -> Result<Value, RuntimeError> {
        let function = self.get_function(function)?;

        if function.parameters.len() != arguments.len() {
            return Err(RuntimeError::InvalidArgumentCount {
                expected: function.parameters.len(),
                found: arguments.len(),
            });
        }

        let mut local_variables = HashMap::new();
        for (param, arg) in function.parameters.iter().zip(arguments) {
            let value = self.execute(arg.clone())?;
            local_variables.insert(param.clone(), value);
        }

        let mut scope = Scope::new(self.vm);
        scope.variables.extend(local_variables);

        let result = scope.execute(function.body)?;

        Ok(result)
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

    pub fn set_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Result<FunctionDef, RuntimeError> {
        self.functions
            .get(name)
            .cloned()
            .ok_or(RuntimeError::UndefinedFunction(name.to_string()))
    }

    pub fn call_stack(&self) -> &[String] {
        &self.call_stack
    }
}
