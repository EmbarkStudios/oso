// The build will fail on stable, but traces will still be printed
// #![feature(trace_macros)]
// trace_macros!(true);

/// Helper macros to create AST types
///
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::rules::*;
use crate::terms::*;

pub const ORD: Ordering = Ordering::SeqCst;
pub static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[macro_export]
macro_rules! value {
    ([$($args:expr),*]) => {
        $crate::terms::Value::List(vec![
            $($crate::term!($crate::value!($args))),*
        ])
    };
    ($arg:expr) => {
        $crate::macros::TestHelper::<$crate::terms::Value>::from($arg).0
    };
}

#[macro_export]
macro_rules! values {
    ($([$($args:expr),*]),*) => {
        vec![$($crate::values!($($args),*)),*]
    };
    ($($args:expr),*) => {
        vec![$($crate::value!($args)),*]
    };
}

#[macro_export]
macro_rules! term {
    ($($expr:tt)*) => {
        $crate::macros::TestHelper::<$crate::terms::Term>::from($crate::value!($($expr)*)).0
    };
}

#[macro_export]
macro_rules! pattern {
    ($arg:expr) => {
        $crate::macros::TestHelper::<$crate::terms::Pattern>::from($arg).0
    };
}

#[macro_export]
macro_rules! param {
    ($($tt:tt)*) => {
        $crate::macros::TestHelper::<$crate::rules::Parameter>::from($($tt)*).0
    };
}

#[macro_export]
macro_rules! instance {
    ($instance:expr) => {
        $crate::terms::InstanceLiteral {
            tag: $crate::sym!($instance),
            fields: $crate::terms::Dictionary::new(),
        }
    };
    ($tag:expr, $fields:expr) => {
        $crate::terms::InstanceLiteral {
            tag: $crate::sym!($tag),
            fields: $crate::macros::TestHelper::<$crate::terms::Dictionary>::from($fields).0,
        }
    };
}

#[macro_export]
macro_rules! sym {
    ($arg:expr) => {
        $crate::macros::TestHelper::<$crate::terms::Symbol>::from($arg).0
    };
}

#[macro_export]
macro_rules! var {
    ($arg:expr) => {
        $crate::macros::TestHelper::<$crate::terms::Term>::from(
            $crate::macros::TestHelper::<$crate::terms::Value>::from($crate::sym!($arg)).0,
        )
        .0
    };
}

#[macro_export]
macro_rules! string {
    ($arg:expr) => {
        $crate::terms::Value::String($arg.into())
    };
}

#[macro_export]
macro_rules! str {
    ($arg:expr) => {
        $crate::macros::TestHelper::<$crate::terms::Term>::from($crate::string!($arg)).0
    };
}

// TODO: support kwargs
#[macro_export]
macro_rules! call {
    ($name:expr) => {
        $crate::terms::Call {
            name: $crate::sym!($name),
            args: vec![],
            kwargs: None
        }
    };
    ($name:expr, [$($args:expr),*]) => {
        $crate::terms::Call {
            name: $crate::sym!($name),
            args: vec![
                $($crate::term!($args)),*
            ],
            kwargs: None
        }
    };
    ($name:expr, [$($args:expr),*], $fields:expr) => {
        $crate::terms::Call {
            name: $crate::sym!($name),
            args: vec![
                $($crate::term!($args)),*
            ],
            kwargs: Some($fields)
        }
    };
}

#[macro_export]
macro_rules! op {
    ($op_type:ident, $($args:expr),+) => {
        $crate::terms::Operation {
            operator: $crate::terms::Operator::$op_type,
            args: vec![$($args),+]
        }
    };
    ($op_type:ident) => {
        $crate::terms::Operation {
            operator: $crate::terms::Operator::$op_type,
            args: vec![]
        }
    };
}

#[macro_export]
macro_rules! dict {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Dictionary>::from($arg).0
    };
}

/// Builds a list of arguments in reverse order
/// Arguments of the form `foo; bar` get built into foo specialized on bar
/// Otherwise, the argument is built depending on the type (symbols become names,
/// terms become specializers).
#[macro_export]
macro_rules! args {
    () => {
        vec![]
    };
    // this is gross: maybe match a <comma plus trailing tokens>
    ($name:expr $(, $($tt:tt)*)?) => {{
        let mut v = args!($($($tt)*)?);
        v.push($crate::param!($crate::value!($name)));
        v
    }};
    ($name:expr ; $spec:expr $(, $($tt:tt)*)?) => {{
        let mut v = $crate::args!($($($tt)*)?);
        v.push($crate::param!(($crate::sym!($name), $crate::term!($spec))));
        v
    }};
}

#[macro_export]
macro_rules! rule {
    ($name:expr, [$($args:tt)*] => $($body:expr),+) => {{
        let mut params = $crate::args!($($args)*);
        params.reverse();
        Rule {
            name: $crate::sym!($name),
            params,
            body: $crate::term!(op!(And, $($crate::term!($body)),+)),
            source_info: $crate::sources::SourceInfo::Test,
            required: false,
        }}
    };
    ($name:expr, [$($args:tt)*]) => {{
        let mut params = $crate::args!($($args)*);
        params.reverse();
        Rule {
            name: $crate::sym!($name),
            params,
            body: $crate::term!($crate::op!(And)),
            source_info: $crate::sources::SourceInfo::Test,
            required: false,
        }
    }};
    // this macro variant is used exclusively to create rule *types*
    // TODO: @patrickod break into specific-purpose rule_type! macro and RuleType struct
    ($name:expr, [$($args:tt)*], $required:expr) => {{
        let mut params = $crate::args!($($args)*);
        params.reverse();
        Rule {
            name: $crate::sym!($name),
            params,
            body: $crate::term!($crate::op!(And)),
            source_info: $crate::sources::SourceInfo::Test,
            required: $required,
        }
    }};
}

/// Special struct which is way more eager at implementing `From`
/// for a bunch of things, so that in the macros we can use `TestHelper<Term>::from`
/// and try and convert things as often as possible.
pub struct TestHelper<T>(pub T);

impl<T> From<T> for TestHelper<T> {
    fn from(other: T) -> Self {
        Self(other)
    }
}

impl From<Value> for TestHelper<Term> {
    fn from(other: Value) -> Self {
        Self(Term::from(other))
    }
}

impl From<u64> for ExternalInstance {
    fn from(instance_id: u64) -> Self {
        ExternalInstance {
            instance_id,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        }
    }
}

// TODO change this
// TODO(gj): TODONE?
impl From<(Symbol, Term)> for TestHelper<Parameter> {
    fn from(arg: (Symbol, Term)) -> Self {
        let specializer = match arg.1.value().clone() {
            Value::Dictionary(dict) => value!(pattern!(dict)),
            v => v,
        };
        Self(Parameter {
            parameter: arg.1.clone_with_value(Value::Variable(arg.0)),
            specializer: Some(term!(specializer)),
        })
    }
}

impl From<Value> for TestHelper<Parameter> {
    /// Convert a Value to a parameter.  If the value is a symbol,
    /// it is used as the parameter name. Otherwise it is assumed to be
    /// a specializer.
    fn from(name: Value) -> Self {
        Self(Parameter {
            parameter: Term::from(name),
            specializer: None,
        })
    }
}

impl<S: AsRef<str>> From<S> for TestHelper<Symbol> {
    fn from(other: S) -> Self {
        Self(Symbol(other.as_ref().to_string()))
    }
}

impl From<BTreeMap<Symbol, Term>> for TestHelper<Dictionary> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Dictionary { fields: other })
    }
}

impl From<i64> for TestHelper<Value> {
    fn from(other: i64) -> Self {
        Self(Value::Number(other.into()))
    }
}

impl From<f64> for TestHelper<Value> {
    fn from(other: f64) -> Self {
        Self(Value::Number(other.into()))
    }
}

impl From<&str> for TestHelper<Value> {
    fn from(other: &str) -> Self {
        Self(Value::String(other.to_string()))
    }
}

impl From<bool> for TestHelper<Value> {
    fn from(other: bool) -> Self {
        Self(Value::Boolean(other))
    }
}

impl From<InstanceLiteral> for TestHelper<Value> {
    fn from(other: InstanceLiteral) -> Self {
        Self(Value::Pattern(Pattern::Instance(other)))
    }
}
impl From<Call> for TestHelper<Value> {
    fn from(other: Call) -> Self {
        Self(Value::Call(other))
    }
}
impl From<Pattern> for TestHelper<Value> {
    fn from(other: Pattern) -> Self {
        Self(Value::Pattern(other))
    }
}
impl From<Operation> for TestHelper<Value> {
    fn from(other: Operation) -> Self {
        Self(Value::Expression(other))
    }
}
impl From<TermList> for TestHelper<Value> {
    fn from(other: TermList) -> Self {
        Self(Value::List(other))
    }
}
impl From<Symbol> for TestHelper<Value> {
    fn from(other: Symbol) -> Self {
        Self(Value::Variable(other))
    }
}
impl From<BTreeMap<Symbol, Term>> for TestHelper<Value> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Value::Dictionary(Dictionary { fields: other }))
    }
}

impl From<Dictionary> for TestHelper<Pattern> {
    fn from(other: Dictionary) -> Self {
        Self(Pattern::Dictionary(other))
    }
}
impl From<BTreeMap<Symbol, Term>> for TestHelper<Pattern> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Pattern::Dictionary(dict!(other)))
    }
}
impl From<InstanceLiteral> for TestHelper<Pattern> {
    fn from(other: InstanceLiteral) -> Self {
        Self(Pattern::Instance(other))
    }
}
impl From<Pattern> for TestHelper<Term> {
    fn from(other: Pattern) -> Self {
        Self(Term::from(value!(other)))
    }
}
