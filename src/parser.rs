use std::collections::HashMap;

use stoml::{Array, Value};

use crate::error::{Error, Result};
use crate::{Arg, ArgType, Matches};

/// Internal argument parser
pub struct ArgParser<'a> {
    /// Map from short flag to arg index
    short_map: HashMap<char, usize>,
    /// Map from long flag to arg index
    long_map: HashMap<String, usize>,
    /// Positional arguments in order
    positionals: Vec<usize>,
    /// Reference to argument definitions
    args: &'a [Arg],
}

impl<'a> ArgParser<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let mut short_map = HashMap::new();
        let mut long_map = HashMap::new();
        let mut positionals = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            if arg.positional {
                positionals.push(i);
            } else {
                if let Some(c) = arg.short {
                    short_map.insert(c, i);
                }
                if let Some(ref l) = arg.long {
                    long_map.insert(l.clone(), i);
                }
            }
        }

        // Sort positionals by their position
        positionals.sort_by_key(|&i| args[i].position);

        Self {
            short_map,
            long_map,
            positionals,
            args,
        }
    }

    pub fn parse(&self, args: Vec<String>) -> Result<Matches> {
        let mut matches = Matches::new();
        let mut args_iter = args.into_iter().peekable();
        let mut positional_index = 0;
        let mut seen_double_dash = false;

        while let Some(arg) = args_iter.next() {
            // After --, everything is a remaining argument
            if seen_double_dash {
                matches.remaining.push(arg);
                continue;
            }

            // Check for --
            if arg == "--" {
                seen_double_dash = true;
                continue;
            }

            // Long flag
            if let Some(rest) = arg.strip_prefix("--") {
                // Check for --no-flag syntax
                if let Some(flag_name) = rest.strip_prefix("no-")
                    && let Some(&idx) = self.long_map.get(flag_name)
                {
                    let arg_def = &self.args[idx];
                    if arg_def.arg_type == ArgType::Bool {
                        matches
                            .values
                            .insert(arg_def.name.clone(), Value::Boolean(false));
                        continue;
                    }
                }

                // Check for --flag=value syntax
                let (flag_name, inline_value) = if let Some(pos) = rest.find('=') {
                    (&rest[..pos], Some(&rest[pos + 1..]))
                } else {
                    (rest, None)
                };

                if let Some(&idx) = self.long_map.get(flag_name) {
                    self.handle_flag(idx, inline_value, &mut args_iter, &mut matches)?;
                } else {
                    return Err(Error::UnknownFlag {
                        flag: format!("--{}", flag_name),
                    });
                }
            }
            // Short flag(s)
            else if let Some(rest) = arg.strip_prefix('-') {
                if rest.is_empty() {
                    // Bare "-" is treated as a positional
                    self.handle_positional("-".to_string(), positional_index, &mut matches)?;
                    positional_index += 1;
                    continue;
                }

                let chars: Vec<char> = rest.chars().collect();
                let mut i = 0;

                while i < chars.len() {
                    let c = chars[i];

                    if let Some(&idx) = self.short_map.get(&c) {
                        let arg_def = &self.args[idx];

                        match arg_def.arg_type {
                            ArgType::Bool => {
                                matches
                                    .values
                                    .insert(arg_def.name.clone(), Value::Boolean(true));
                                i += 1;
                            }
                            ArgType::Count => {
                                let current = matches
                                    .values
                                    .get(&arg_def.name)
                                    .and_then(|v| v.as_integer())
                                    .unwrap_or(0);
                                matches
                                    .values
                                    .insert(arg_def.name.clone(), Value::Integer(current + 1));
                                i += 1;
                            }
                            _ => {
                                // Value-taking flag
                                // Check if the rest of the chars form the value
                                if i + 1 < chars.len() {
                                    let value: String = chars[i + 1..].iter().collect();
                                    self.set_value(idx, &value, &mut matches)?;
                                    break;
                                } else {
                                    // Value is in the next argument
                                    let value =
                                        args_iter.next().ok_or_else(|| Error::MissingValue {
                                            name: arg_def.name.clone(),
                                        })?;
                                    self.set_value(idx, &value, &mut matches)?;
                                    i += 1;
                                }
                            }
                        }
                    } else {
                        return Err(Error::UnknownFlag {
                            flag: format!("-{}", c),
                        });
                    }
                }
            }
            // Positional argument
            else {
                self.handle_positional(arg, positional_index, &mut matches)?;
                positional_index += 1;
            }
        }

        Ok(matches)
    }

    fn handle_flag(
        &self,
        idx: usize,
        inline_value: Option<&str>,
        args_iter: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
        matches: &mut Matches,
    ) -> Result<()> {
        let arg_def = &self.args[idx];

        match arg_def.arg_type {
            ArgType::Bool => {
                if let Some(inline_val) = inline_value {
                    // --flag=value for bool - parse the value
                    let lower = inline_val.to_lowercase();
                    let b = lower == "true" || lower == "1" || lower == "yes";
                    matches
                        .values
                        .insert(arg_def.name.clone(), Value::Boolean(b));
                } else {
                    matches
                        .values
                        .insert(arg_def.name.clone(), Value::Boolean(true));
                }
            }
            ArgType::Count => {
                let current = matches
                    .values
                    .get(&arg_def.name)
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0);
                matches
                    .values
                    .insert(arg_def.name.clone(), Value::Integer(current + 1));
            }
            _ => {
                let value = if let Some(v) = inline_value {
                    v.to_string()
                } else {
                    args_iter.next().ok_or_else(|| Error::MissingValue {
                        name: arg_def.name.clone(),
                    })?
                };
                self.set_value(idx, &value, matches)?;
            }
        }

        Ok(())
    }

    fn handle_positional(&self, value: String, index: usize, matches: &mut Matches) -> Result<()> {
        // Find the appropriate positional argument
        if index < self.positionals.len() {
            let arg_idx = self.positionals[index];
            let arg_def = &self.args[arg_idx];

            if arg_def.variadic {
                // Variadic: add to array
                let arr = matches
                    .values
                    .entry(arg_def.name.clone())
                    .or_insert_with(|| Value::Array(Array::new()));
                if let Value::Array(a) = arr {
                    a.push(self.parse_value_as_type(&value, ArgType::String)?);
                }
            } else {
                matches.values.insert(
                    arg_def.name.clone(),
                    self.parse_value_as_type(&value, arg_def.arg_type)?,
                );
            }
        } else {
            // Check if the last positional is variadic
            if let Some(&last_idx) = self.positionals.last() {
                let last_arg = &self.args[last_idx];
                if last_arg.variadic {
                    let arr = matches
                        .values
                        .entry(last_arg.name.clone())
                        .or_insert_with(|| Value::Array(Array::new()));
                    if let Value::Array(a) = arr {
                        a.push(self.parse_value_as_type(&value, ArgType::String)?);
                    }
                    return Ok(());
                }
            }

            return Err(Error::TooManyPositional {
                max: self.positionals.len(),
                got: index + 1,
            });
        }

        Ok(())
    }

    fn set_value(&self, idx: usize, value: &str, matches: &mut Matches) -> Result<()> {
        let arg_def = &self.args[idx];

        match arg_def.arg_type {
            ArgType::Array => {
                // Arrays accumulate multiple values
                let arr = matches
                    .values
                    .entry(arg_def.name.clone())
                    .or_insert_with(|| Value::Array(Array::new()));
                if let Value::Array(a) = arr {
                    a.push(self.parse_value_as_type(value, ArgType::String)?);
                }
            }
            _ => {
                // Non-arrays: check for duplicates (unless it's a count)
                if matches.values.contains_key(&arg_def.name) && arg_def.arg_type != ArgType::Count
                {
                    return Err(Error::DuplicateValue {
                        name: arg_def.name.clone(),
                    });
                }
                matches.values.insert(
                    arg_def.name.clone(),
                    self.parse_value_as_type(value, arg_def.arg_type)?,
                );
            }
        }

        Ok(())
    }

    fn parse_value_as_type(&self, value: &str, arg_type: ArgType) -> Result<Value> {
        match arg_type {
            ArgType::String => Ok(Value::String(value.to_string())),
            ArgType::Integer => {
                value
                    .parse::<i64>()
                    .map(Value::Integer)
                    .map_err(|_| Error::InvalidValue {
                        name: String::new(),
                        value: value.to_string(),
                        expected: "an integer",
                    })
            }
            ArgType::Float => {
                value
                    .parse::<f64>()
                    .map(Value::Float)
                    .map_err(|_| Error::InvalidValue {
                        name: String::new(),
                        value: value.to_string(),
                        expected: "a number",
                    })
            }
            ArgType::Bool => {
                let lower = value.to_lowercase();
                Ok(Value::Boolean(
                    lower == "true" || lower == "1" || lower == "yes",
                ))
            }
            ArgType::Count => {
                value
                    .parse::<i64>()
                    .map(Value::Integer)
                    .map_err(|_| Error::InvalidValue {
                        name: String::new(),
                        value: value.to_string(),
                        expected: "an integer",
                    })
            }
            ArgType::Array => Ok(Value::String(value.to_string())),
        }
    }
}
