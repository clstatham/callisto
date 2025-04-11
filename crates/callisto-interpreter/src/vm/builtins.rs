use crate::parser::syntax::{Syntax, SyntaxType};

use super::{FunctionDef, RuntimeError, Scope, value::Value};

impl Scope<'_> {
    pub fn execute_builtin_function(
        &mut self,
        function: &str,
        arguments: &[Syntax],
    ) -> Result<Value, RuntimeError> {
        match function {
            "define" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                if let Syntax::Identifier(name) = &arguments[0] {
                    let value = self.execute(arguments[1].clone())?;
                    self.set_variable(name.clone(), value);
                    Ok(Value::Null)
                } else {
                    Err(RuntimeError::SyntaxError {
                        expected: SyntaxType::Identifier,
                        found: arguments[0].syntax_type(),
                    })
                }
            }
            "do" => {
                if arguments.is_empty() {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    });
                }
                let mut result = Value::Null;
                for arg in arguments {
                    result = self.execute(arg.clone())?;
                }
                Ok(result)
            }
            "func" => {
                if arguments.len() != 3 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 3,
                        found: arguments.len(),
                    });
                }
                if let Syntax::Identifier(name) = &arguments[0] {
                    if let Syntax::List(params) = &arguments[1] {
                        let mut parameters = Vec::new();
                        for param in params {
                            if let Syntax::Identifier(param_name) = param {
                                parameters.push(param_name.clone());
                            } else {
                                return Err(RuntimeError::SyntaxError {
                                    expected: SyntaxType::Identifier,
                                    found: param.syntax_type(),
                                });
                            }
                        }
                        let function_def = FunctionDef {
                            name: name.clone(),
                            parameters,
                            body: arguments[2].clone(),
                        };
                        self.set_function(function_def);
                        Ok(Value::Null)
                    } else {
                        Err(RuntimeError::SyntaxError {
                            expected: SyntaxType::List,
                            found: arguments[1].syntax_type(),
                        })
                    }
                } else {
                    Err(RuntimeError::SyntaxError {
                        expected: SyntaxType::Identifier,
                        found: arguments[0].syntax_type(),
                    })
                }
            }
            "let" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                if let Syntax::List(bindings) = &arguments[0] {
                    for binding in bindings {
                        if let Syntax::List(pair) = binding {
                            if pair.len() != 2 {
                                return Err(RuntimeError::InvalidArgumentCount {
                                    expected: 2,
                                    found: pair.len(),
                                });
                            }
                            if let Syntax::Identifier(name) = &pair[0] {
                                let value = self.execute(pair[1].clone())?;
                                self.set_variable(name.clone(), value);
                            } else {
                                return Err(RuntimeError::SyntaxError {
                                    expected: SyntaxType::Identifier,
                                    found: pair[0].syntax_type(),
                                });
                            }
                        } else {
                            return Err(RuntimeError::SyntaxError {
                                expected: SyntaxType::List,
                                found: binding.syntax_type(),
                            });
                        }
                    }
                    self.execute(arguments[1].clone())
                } else {
                    Err(RuntimeError::SyntaxError {
                        expected: SyntaxType::List,
                        found: arguments[0].syntax_type(),
                    })
                }
            }
            "apply" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                if let Syntax::Identifier(name) | Syntax::Operator(name) = &arguments[0] {
                    if let Syntax::List(args) = &arguments[1] {
                        self.call_stack.push(name.clone());
                        let result = self.execute_builtin_function(name, args);
                        self.call_stack.pop();
                        match result {
                            Ok(value) => return Ok(value),
                            Err(RuntimeError::UndefinedFunction(_)) => {}
                            Err(e) => return Err(e),
                        }

                        self.call_stack.push(name.clone());
                        let result = self.execute_function(name, args);
                        self.call_stack.pop();
                        match result {
                            Ok(value) => return Ok(value),
                            Err(RuntimeError::UndefinedFunction(_)) => {}
                            Err(e) => return Err(e),
                        }

                        Err(RuntimeError::UndefinedIdentifier(name.clone()))
                    } else {
                        Err(RuntimeError::SyntaxError {
                            expected: SyntaxType::List,
                            found: arguments[1].syntax_type(),
                        })
                    }
                } else {
                    Err(RuntimeError::SyntaxError {
                        expected: SyntaxType::Identifier,
                        found: arguments[0].syntax_type(),
                    })
                }
            }
            "+" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                let mut result = self.execute(arguments[0].clone())?;
                for arg in &arguments[1..] {
                    let value = self.execute(arg.clone())?;
                    result = result.add(&value)?;
                }
                Ok(result)
            }
            "-" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                a.sub(&b)
            }
            "*" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                let mut result = self.execute(arguments[0].clone())?;
                for arg in &arguments[1..] {
                    let value = self.execute(arg.clone())?;
                    result = result.mul(&value)?;
                }
                Ok(result)
            }
            "/" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }
                let a = self.execute(arguments[0].clone())?;
                let b = self.execute(arguments[1].clone())?;
                a.div(&b)
            }

            _ => Err(RuntimeError::UndefinedFunction(function.to_string())),
        }
    }
}
