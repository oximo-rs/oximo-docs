+++
title = "Generalized Disjunctive Programming"
description = "Model logical decisions and conditional rows, then reformulate them to MILP/MINLP."
weight = 8

[extra]
math = true
+++

Generalized Disjunctive Programming (GDP) expresses optimization problems with
logical decisions and groups of conditional rows. For example, choosing
a small unit activates its capacity constraints, while choosing a large unit
activates a different set of constraints.

The optional `oximo-gdp` crate adds GDP modeling to the ordinary `Model`.
Reformulation converts the logical and conditional components into algebraic
constraints that a compatible solver can handle. Choose from
[Big-M and Hull](#reformulations-available). For solving, prefer
`solver.with_gdp(...).solve(...)`: it reformulates the model and returns results
for your declared variables and Hull activations. Use explicit reformulation to
export or inspect the transformed model, or to create an independent copy.

## Enable GDP

Enable the `gdp` feature on `oximo`:

```toml
[dependencies]
oximo = { version = "0.7.0", features = ["gdp", "highs"] }
```

In this case, `gdp` provides modeling and reformulation while `highs` provides
a solver for the linear example below. Import `oximo::prelude::*` for the macros, types, and
`GdpModelExt` methods.

## Terminology

This document uses the following terms:

| Term               | Meaning                                                                                                                                              | API                                                                  |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Boolean variable   | A logical variable \\(Y\\), with value `true` or `false`.                                                                                            | `BooleanHandle`, declared with `boolean_variable!` or by `disjunct!` |
| Indicator          | The Boolean decision that activates a disjunct or conditional row.                                                                                   | `handle.indicator()`                                                 |
| Binary counterpart | The algebraic variable \\(y \\in \\{0,1\\}\\) corresponding to \\(Y\\). Use it in arithmetic and numeric result queries.                             | `handle.binary()`                                                    |
| Conditional row    | An algebraic relation that applies only when its indicator is true. “Disjunct constraint” is the API's name for a conditional row.                   | `DisjunctConstraintHandle`, `disjunct_constraint!`                   |
| Disjunct           | A block of conditional rows sharing one indicator.                                                                                                   | `DisjunctHandle`, `disjunct!`                                        |
| Branch             | One member of a disjunction: a disjunct handle or a separately declared Boolean decision. A Boolean branch may have conditional rows attached to it. | An element of the `branches` argument                                |
| Disjunction        | A selection rule over a collection of branches.                                                                                                      | `DisjunctionHandle`, `disjunction!`                                  |

The relationship between \\(Y\\) and \\(y\\) is shown in [Boolean decisions](#boolean-decisions).
Use **conditional row** for the relation and **disjunct constraint** when referring to its API.

## Boolean decisions

A Boolean variable represents a logical decision:

$$
Y\_i \\in \\{\\mathrm{true},\\mathrm{false}\\}.
$$

It is distinct from an algebraic variable such as a flow \\(x \\in \\mathbb{R}\\).
For example, \\(Y\_i = \\mathrm{true}\\) can mean that operating mode \\(i\\) is selected.

Boolean variables are intended for logical expressions and do not participate
directly in algebraic expressions. For example, \\(Y\_i + 1\\) is undefined,
so an objective such as \\(q - 2Y\_{\\mathrm{large}}\\) is also undefined.
To model a conditional operating cost, introduce an algebraic
variable \\(c\\) and use a disjunction. Here, \\(\\underline{\\bigvee}\\) denotes
exactly-one selection, the default rule for `disjunction!`.
Plain \\(\\bigvee\\) denotes inclusive OR, where at least one block must be selected.

$$
\\begin{aligned}
\\max\\quad & q-c \\\\
\\text{subject to}\\quad &
\\left[\\begin{array}{c}
Y\_{\\mathrm{small}} \\\\
c = 0
\\end{array}\\right]
\\underline{\\bigvee}
\\left[\\begin{array}{c}
Y\_{\\mathrm{large}} \\\\
c = 2
\\end{array}\\right], \\\\
& 0 \\le c \\le 2.
\\end{aligned}
$$

The full formulation appears in [Disjunct blocks](#disjunct-blocks) below.

Declare Boolean decisions independently when you want to use them in logical
constraints or attach them to individual conditional rows:

```rust
use oximo::prelude::*;

let model = Model::new("Boolean decisions");
boolean_variable!(model, selected);
boolean_variable!(model, mode[i in 0..2]);
```

A disjunct block also declares its own Boolean decision. The GDP formulation
uses \\(Y\_i\\). Its algebraic counterpart is a binary variable \\(y\_i\\) satisfying

$$
y\_i =
\\begin{cases}
1, & Y\_i = \\mathrm{true}, \\\\
0, & Y\_i = \\mathrm{false}.
\\end{cases}
$$

`handle.binary()` exposes this numeric counterpart for algebraic expressions and
numeric result queries. Read a Boolean selection with `result.boolean_value_of(handle)`.
Arithmetic on \\(y\_i\\) is valid because it is a binary
algebraic variable. This explicit mapping does not make \\(Y\_i\\) itself numeric.

## Disjunct blocks

A disjunct groups conditional rows under one indicator. For a vector of
algebraic variables \\(x\\), write disjunct \\(i\\) as

$$
D\_i = \\left[\\begin{array}{c}
Y\_i \\\\
h\_i(x) \\le 0
\\end{array}\\right].
$$

Here \\(Y\_i\\) is the Boolean decision and \\(h\_i(x) \\le 0\\) represents all the
conditional rows in its block. The block means \\(Y\_i \\Rightarrow h\_i(x) \\le 0\\).
Its rows apply when \\(Y\_i\\) is true. When \\(Y\_i\\) is false, that block imposes
no restrictions on \\(x\\). Equalities and ranges can also appear inside a disjunct block.

A disjunction specifies which blocks must be selected. The default exactly-one
rule selects either the small or the large unit:

$$
\\begin{aligned}
\\max\\quad & q-c \\\\
\\text{subject to}\\quad &
\\left[\\begin{array}{c}
Y\_{\\mathrm{small}} \\\\
q \\le 3 \\\\
c = 0
\\end{array}\\right]
\\underline{\\bigvee}
\\left[\\begin{array}{c}
Y\_{\\mathrm{large}} \\\\
5 \\le q \\le 8 \\\\
c = 2
\\end{array}\\right] \\\\
& 0 \\le q \\le 10, \\qquad 0 \\le c \\le 2, \\\\
& Y\_{\\mathrm{small}},Y\_{\\mathrm{large}}
  \\in \\{\\mathrm{true},\\mathrm{false}\\}.
\\end{aligned}
$$

\\(q\\) is flow and \\(c\\) is operating cost. The objective and variable bounds are
global; capacity, minimum flow, and fixed cost belong to their respective blocks.

```rust
use oximo::prelude::*;
use oximo::{HighsOptions, solvers::Highs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("unit selection");
    variable!(model, 0.0 <= flow <= 10.0);
    variable!(model, 0.0 <= cost <= 2.0);

    let small = disjunct!(model, small, |d| {
        constraint!(d, capacity, flow <= 3.0);
        constraint!(d, fixed_cost, cost == 0.0);
    });
    let large = disjunct!(model, large, |d| {
        constraint!(d, operating_range, 5.0 <= flow <= 8.0);
        constraint!(d, fixed_cost, cost == 2.0);
    });
    let choice = disjunction!(model, unit, [small, large]);
    objective!(model, Max, flow - cost);

    let result = Highs
        .with_gdp(BigM::default())
        .solve(&model, &HighsOptions::default())?;

    println!("objective: {:?}", result.objective());
    println!("flow: {:?}", result.value_of(flow)?);
    println!("cost: {:?}", result.value_of(cost)?);
    let selected = result.boolean_value_of(large)?;
    println!("large unit selected: {selected:?}");
    Ok(())
}
```

Expected output, apart from any solver log:

```text
objective: Some(6.0)
flow: Some(8.0)
cost: Some(2.0)
large unit selected: Some(true)
```

The solution selects the large unit; its conditional rows enforce the cost.
`boolean_value_of` returns `Result<Option<bool>, BooleanValueError>`. The
helper uses a fixed absolute tolerance of `1e-5`. Values within `1e-5` of zero or
one become `Some(false)` or `Some(true)`. A missing solution or value stays
`None`, and values farther from both, or non-finite values, return
`BooleanValueError::NonBooleanValue`.

To inspect fractional selections from an LP relaxation, use the numeric query
`result.value_of(large.binary())?`. See [Results](../results/) for checking solve
status and solution availability.

Declare algebraic variables and objectives on the global model. The block
receiver supports scalar algebraic constraints, including nonlinear expressions,
ranges, indexed families, and `sum!`. **Conditional variables, objectives, cones,
native indicators, and SOS constraints are not supported.**

## Boolean-tagged constraints

`disjunct_constraint!` attaches a conditional row to an existing Boolean decision,
so you can add conditions without declaring a disjunct block. Each row expresses an
implication \\(Y\_i \\Rightarrow l\_i \\le f\_i(x) \\le u\_i\\).

For the nonlinear example below, \\(L\_0 = 2\\) and \\(L\_1 = 4\\):

$$
\\begin{aligned}
\\max\\quad & x \\\\
\\text{subject to}\\quad & Y\_i \\Rightarrow e^x \\le L\_i,
  \\qquad i \\in \\{0,1\\}, \\\\
& \\operatorname{ExactlyOne}(Y\_0,Y\_1), \\\\
& 0 \\le x \\le 3, \\qquad
  Y\_0,Y\_1 \\in \\{\\mathrm{true},\\mathrm{false}\\}.
\\end{aligned}
$$

\\(\\operatorname{ExactlyOne}\\) means exactly one argument is true:

```rust
use oximo::prelude::*;

let model = Model::new("tagged choices");
variable!(model, 0.0 <= x <= 3.0);
boolean_variable!(model, selected[i in 0..2]);

let limits = [2.0, 4.0];
let rows = disjunct_constraint!(model, selected[i],
    capacity[i in 0..2], x.exp() <= limits[i]);
let branches = [selected[0], selected[1]];
let choice = disjunction!(model, choice, branches);
objective!(model, Max, x);

model.reformulate_gdp(BigM::default())?;
```

This produces a mixed-integer nonlinear model. Choose a backend with support
for the resulting expressions; see [Solvers](../solvers/).

## Macro grammar

In this table, `m` is the model or disjunct context, `label` is a name written as
a Rust identifier, and `name = expr` computes the registered name at runtime.
`body` is a closure such as `|d| { constraint!(d, x <= 2.0); }`, `relation` is an
algebraic comparison or range, and `formula` is a Boolean expression.
The forms in each cell are alternatives.

| Macro                  | Accepted forms                                                                                                                                                                                                                                  | Binding or return value                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `boolean_variable!`    | `boolean_variable!(m, decision)`<br>`boolean_variable!(m, decision[i in domain])`                                                                                                                                                               | Binds `decision` locally to a Boolean handle or indexed family. A name is required; anonymous and computed-name forms are not accepted. |
| `disjunct!`            | `disjunct!(m, body)`<br>`disjunct!(m, label, body)`<br>`disjunct!(m, name = expr, body)`<br>`disjunct!(m, label[i in domain], body)`                                                                                                            | Returns a disjunct handle or indexed family.                                                                                            |
| `disjunction!`         | `disjunction!(m, branches)`<br>`disjunction!(m, label, branches)`<br>`disjunction!(m, name = expr, branches)`<br>`disjunction!(m, label[i in domain], branches)`                                                                                | Returns a disjunction handle or indexed family. Each form also accepts the options below.                                               |
| `disjunct_constraint!` | `disjunct_constraint!(m, indicator, relation)`<br>`disjunct_constraint!(m, indicator, label, relation)`<br>`disjunct_constraint!(m, indicator, name = expr, relation)`<br>`disjunct_constraint!(m, indicator[i], label[i in domain], relation)` | Returns a conditional-row handle, range handles, or an indexed family, depending on the relation.                                       |
| `logical_constraint!`  | `logical_constraint!(m, formula)`<br>`logical_constraint!(m, label, formula)`<br>`logical_constraint!(m, name = expr, formula)`<br>`logical_constraint!(m, label[i in domain], formula)`                                                        | Returns a logical-constraint handle or indexed family.                                                                                  |

The macros that return handles do **not** bind their registered names as Rust
variables. You need to capture the result explicitly, for example
`let choice = disjunction!(model, unit, [small, large]);`. Here, `unit` names the
disjunction in the model, while `choice` is the Rust variable holding its handle.

For `disjunction!`, append a selector (`ExactlyOne` or `AtLeastOne`),
`parent = indicator`, or both to any form. The selector defaults to `ExactlyOne`.
The two options may appear in either order, and each may appear only once.

| Example                                                             | How the arguments are read                                                              |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `disjunction!(model, unit, branches)`                               | `unit` is the name; `branches` is the collection.                                       |
| `disjunction!(model, branches, AtLeastOne)`                         | Anonymous disjunction; `branches` is the collection; `AtLeastOne` is the selector.      |
| `disjunction!(model, branches, parent = present)`                   | Anonymous disjunction; `branches` is the collection; `present` is the parent indicator. |
| `disjunction!(model, unit, branches, AtLeastOne, parent = present)` | Named disjunction with an inclusive selector and a parent.                              |

An indexed name may use multiple domains and a filter:
`label[i in domain, j in other_domain if condition]`. The index variables are
available in the body, indicator, formula, branches, and parent expression as
appropriate. String and tuple keys are supported. Families provide `get`, `iter`,
indexing, and `len`. Use `family.values()` to pass an indexed family to a logical
builder, for example `exactly(1, family.values())`.

## Disjunction semantics

For a branch index set \\(I\\), the default exactly-one disjunction requires

$$
\\underline{\\bigvee}\_{i \\in I} Y\_i = \\mathrm{true}.
$$

The underlined operator means exactly one branch is true, equivalent to
\\(\\operatorname{ExactlyOne}(Y\_i : i \\in I)\\). Supply `AtLeastOne` for
inclusive OR, which requires

$$
\\bigvee\_{i \\in I} Y\_i = \\mathrm{true}.
$$

At least one branch must be true, and several may be true. Every selected
branch enforces all of its conditional rows:

```rust
let branches = [small, large];
disjunction!(model, branches, AtLeastOne);
```

This is an alternative to the exactly-one declaration in the first example.
Each branch can belong to at most one disjunction, so select the desired form
when constructing the model. `ExactlyOne` is also accepted explicitly.

Compare selection rules with logical XOR:

| Form                                                        | Requirement                                                          |
| ----------------------------------------------------------- | -------------------------------------------------------------------- |
| `disjunction!(model, branches)` or an explicit `ExactlyOne` | Exactly one branch is true, for any number of branches.              |
| `disjunction!(model, branches, AtLeastOne)`                 | One or more branches are true; several may be selected together.     |
| `a ^ b`                                                     | Binary XOR: exactly one of these **two** formulas is true.           |
| `a ^ b ^ c`                                                 | Chained XOR: an odd number of formulas is true, including all three. |

For three or more decisions, use `exactly(1, terms)` for an exactly-one logical
constraint. Chaining `^` would also admit three, five, or more true terms.

## Logical constraints

A logical constraint asserts that a Boolean formula \\(\\Omega(Y)\\) is true:

$$
\\Omega(Y) = \\mathrm{true}.
$$

For example, \\(Y\_a \\Rightarrow Y\_b\\) means selecting \\(a\\) requires selecting \\(b\\).
Logical constraints connect decisions without writing these relationships
manually as algebraic inequalities. With \\(A\\) and \\(B\\) denoting Boolean formulas,
and \\(N\\) denoting the number of true terms:

| Mathematical notation      | Rust expression                     | Meaning                        |
| -------------------------- | ----------------------------------- | ------------------------------ |
| \\(\\neg A\\)              | `!a`                                | NOT                            |
| \\(A \\land B\\)           | `a & b`                             | AND                            |
| \\(A \\lor B\\)            | `a \| b`                            | Inclusive OR                   |
| \\(A \\oplus B\\)          | `a ^ b`                             | Exclusive OR of two formulas   |
| \\(A \\Rightarrow B\\)     | `implies(a, b)` or `a.implies(b)`   | If `a`, then `b`               |
| \\(A \\Leftrightarrow B\\) | `iff(a, b)` or `a.equivalent_to(b)` | Both have the same truth value |
| \\(N = k\\)                | `exactly(k, terms)`                 | Exactly `k` terms are true     |
| \\(N \\le k\\)             | `at_most(k, terms)`                 | At most `k` terms are true     |
| \\(N \\ge k\\)             | `at_least(k, terms)`                | At least `k` terms are true    |

Assert a formula with `logical_constraint!`:

```rust
logical_constraint!(model, selected.implies(mode[0]));
```

`logical_and(terms)` and `logical_or(terms)` accept iterators. Terms may be
Boolean handles, Boolean constants, or compound logical expressions, including
cardinality expressions. An empty AND is true, while an empty OR is false.

Use `&` and `|` for model logic, since Rust's `&&` and `||` require actual Rust Booleans.
Parenthesize compound formulas.

## Nested disjunctions

A nested disjunction is selected only when its parent decision \\(P\\) is true.
For exactly-one children \\(Y\_i\\), its meaning is

$$
P \\Rightarrow \\operatorname{ExactlyOne}(Y\_i : i \\in I),
\\qquad Y\_i \\Rightarrow P \\quad \\text{for every } i \\in I.
$$

If \\(P\\) is false, every child is false. If \\(P\\) is true, exactly one child is true.
For inclusive selection, replace \\(\\operatorname{ExactlyOne}\\) with
\\(\\bigvee\_{i \\in I}Y\_i\\). A logical formula declared in the parent's context
expresses \\(P \\Rightarrow \\Omega(Y)\\).

For example, a unit is either present or absent. When present, it chooses one
of two technologies with different flow limits. The inner disjunction sits
inside the present branch.

$$
\\begin{aligned}
&
\\left[\\begin{array}{c}
P \\\\
\\left[\\begin{array}{c}
Y\_0 \\\\
q \\le 3
\\end{array}\\right]
\\underline{\\bigvee}
\\left[\\begin{array}{c}
Y\_1 \\\\
q \\le 8
\\end{array}\\right]
\\end{array}\\right]
\\underline{\\bigvee}
\\left[\\begin{array}{c}
A \\\\
q = 0
\\end{array}\\right], \\\\
& 0 \\le q \\le 8, \\qquad
P,A,Y\_0,Y\_1 \\in \\{\\mathrm{true},\\mathrm{false}\\}.
\\end{aligned}
$$

Here \\(P\\) is `present`, \\(A\\) is `absent`, \\(q\\) is `flow`, and the
children are `technology[0]` and `technology[1]`. Attach them to the parent
with `parent = ...`:

```rust
use oximo::prelude::*;

let model = Model::new("nested technology selection");
variable!(model, 0.0 <= flow <= 8.0);
boolean_variable!(model, present);
boolean_variable!(model, absent);
boolean_variable!(model, technology[i in 0..2]);

disjunct_constraint!(model, absent, no_flow, flow == 0.0);
let limits = [3.0, 8.0];
disjunct_constraint!(model, technology[i],
    capacity[i in 0..2], flow <= limits[i]);

disjunction!(model, unit, [present, absent]);
let branches = [technology[0], technology[1]];
disjunction!(model, branches, parent = present);

model.reformulate_gdp(BigM::default())?;
```

You can also declare child disjuncts and their disjunction directly inside a
parent block. The nesting graph must be acyclic.

## Reformulations available

oximo provides **Big-M** and **Hull** reformulations. Both turn conditional rows
and logical decisions into an algebraic model. The choice affects its size and
continuous relaxation, where binary variables can take fractional values.
`GdpReformulationOptions::default()` selects Big-M.

| Method            | How it works                                                                      | What to consider                                                                                                                   |
| ----------------- | --------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `BigM::default()` | Adds \\(M\\) terms that relax a row when its branch is inactive.                  | Produces a compact model. Needs valid \\(M\\) values; large values can weaken the relaxation and cause numerical difficulties.     |
| `Hull::default()` | Introduces branch-specific variable copies and transforms the rows that use them. | Can strengthen the relaxation, at the cost of more variables and rows. Requires finite lower and upper bounds on copied variables. |

Both methods support affine and nonlinear conditional rows, nested disjunctions,
and standalone conditional rows. Choose a solver that supports the resulting
algebraic expressions.

### Big-M

Big-M is a useful reformulation when model size matters and you can establish
tight, valid bounds. The default method estimates \\(M\\) values from global variable
bounds and uses unconditional affine constraints to tighten those bounds.
For nested groups, it also uses parent and ancestor constraints to tighten local
\\(M\\) values. Both forms of tightening are enabled by default.

The bounds must remain valid in **every feasible mode**, including when a branch
is inactive. Constraints inside a branch do not establish global bounds. An \\(M\\)
that is too small can remove feasible solutions, while a larger valid \\(M\\) may slow solving.

The main options are:

| Option                                                                            | When to use it                                                                                                                               |
| --------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `options.with_big_m(row, BigMValues::upper(value))` or `BigMValues::lower(value)` | Supply a justified \\(M\\) for one row side when you can improve on automatic estimation. Values must be finite and nonnegative.             |
| `BigM::default().with_fallback_big_m(value)`                                      | Supply a justified value for sides where automatic estimation cannot find a finite \\(M\\). It does not replace a finite automatic estimate. |
| `BigM::default().with_hierarchical_tightening(false)`                             | Disable ancestor tightening for nested groups, for example to compare formulations.                                                          |
| `options.with_bound_tightening(false)`                                            | Disable affine bound propagation, including hierarchical propagation. Estimates then rely on declared bounds.                                |

There is no large default \\(M\\). If neither estimation nor your settings supply a
valid value, reformulation returns `MissingBigM`. Nonlinear expressions must also
be defined over the global domain, even when their branch is inactive; supplying
an \\(M\\) does not repair an unsafe domain.

See [Big-M values](#big-m-values) for the generated inequalities, \\(M\\) precedence,
and examples of fixing missing estimates or unsafe domains.

### Hull

Hull is useful when a stronger relaxation can justify a larger algebraic model.
It avoids choosing \\(M\\) values, but every variable that needs a copy must have
**finite lower and upper bounds**. The default affine propagation pass can derive
these bounds from unconditional constraints.

The main options are:

| Option                                                        | Default and purpose                                                                                                                                                                     |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Hull::default().with_inclusive_policy(...)`                  | Rejects `AtLeastOne` groups by default. Enable `HullInclusivePolicy::Enumerate { max_patterns }` to preserve inclusive selection.                                                       |
| `options.with_hull_reference_coordinate(variable, reference)` | References default to zero. Set a finite reference when a nonlinear function is undefined at zero, or to improve coordinate scaling. Pass a bare variable handle from the source model. |
| `Hull::default().with_epsilon(value)`                         | Uses `1e-6` for nonlinear perspective regularization. A custom value must be finite and strictly between zero and one.                                                                  |

A nonlinear reference must make both the reference evaluation and the generated
perspective domain safe. It doesn't need to satisfy the branch's constraints, and a
point inside the source domain is not always safe for the generated expression.
For convex disjunct sets, smaller epsilon values bring the continuous relaxation
closer to the closed perspective.
See [Nonlinear perspectives and domains](#nonlinear-perspectives-and-domains)
for the equations and validation rules.

#### Reference example

Restricted-domain functions may require an explicit reference. For example,
zero is not a valid reference for the logarithm below:

```rust
use oximo::prelude::*;

let model = Model::new("logarithmic Hull");
variable!(model, 1.0 <= flow <= 4.0);
boolean_variable!(model, active);
disjunct_constraint!(model, active, flow.ln() <= 3.0_f64.ln());

let options = GdpReformulationOptions::new(Hull::default())
    .with_hull_reference_coordinate(flow, 1.0);
model.reformulate_gdp(options)?;
```

Use `with_hull_reference_coordinates(...)` to set references for several
variables at once; see [Affine rows and reference coordinates](#affine-rows-and-reference-coordinates).

#### Inclusive selection

Hull requires `ExactlyOne` by default. To preserve `AtLeastOne` semantics,
explicitly enable local activation-pattern enumeration:

```rust
use oximo::prelude::*;

let model = Model::new("inclusive Hull");
variable!(model, 0.0 <= flow <= 4.0);
boolean_variable!(model, left);
boolean_variable!(model, right);
disjunct_constraint!(model, left, flow <= 2.0);
disjunct_constraint!(model, right, flow >= 1.0);
disjunction!(model, [left, right], AtLeastOne);

let hull = Hull::default().with_inclusive_policy(
    HullInclusivePolicy::Enumerate { max_patterns: 3 },
);
model.reformulate_gdp(hull)?;
```

An inclusive group with `n` branches has `2^n - 1` nonempty selection patterns,
so enumeration grows exponentially. The limit applies to each group separately.
For larger inclusive groups, consider Big-M or inspect the expansion with
[`plan_gdp`](#planning-and-expansion-limits) before transforming the model.

## Applying a reformulation

### Select during solving

Prefer `with_gdp` for routine GDP solves. It reformulates pending GDP components
in place, calls the backend with its ordinary options, and returns results for
your declared variables and Hull activations:

```rust
let result = Highs
    .with_gdp(Hull::default())
    .solve(&model, &HighsOptions::default())?;

println!("flow: {:?}", result.value_of(flow)?);
```

Other generated variables and multipliers for generated rows are omitted.
Variables and ordinary constraints added between solves remain available in
later results. The transformation report is available through
`model.gdp_reformulations()`; see [Hull activation queries](#hull-copy-and-activation-reports).

> **Changing data or methods between solves.** Both `with_gdp` and explicit
> reformulation lock bounds and parameter values used by transformed rows. To
> change those inputs or compare methods, keep the source unreformulated and use
> `to_reformulated_gdp_model(...)` for an independent copy. See
> [Lifecycle, cloning, and reports](#lifecycle-cloning-and-reports).

### Per-disjunction configuration

Use `GdpReformulationOptions` to mix methods or change settings for individual
disjunctions. This configuration uses Big-M by default and Hull for `choice`:

```rust
let gdp = GdpReformulationOptions::new(BigM::default())
    .with_method(choice, Hull::default());

let result = Highs
    .with_gdp(gdp)
    .solve(&model, &HighsOptions::default())?;
```

The same configuration can instead be passed to `model.reformulate_gdp(gdp)`
or `model.to_reformulated_gdp_model(gdp)`.

Every disjunction without an override uses the method supplied to `new`,
including any third choice you did not list with `with_method`. Each nested
disjunction uses its own override or that same method; overrides do not inherit
from parents. Standalone conditional rows also use the settings supplied to
`new`. The last override for a handle wins, and foreign-model handles are rejected.

Reformulation processes only pending components. Selecting another method after
an in-place transformation does not replace the existing reformulation. To
compare methods, retain an unreformulated source and create independent copies
with `to_reformulated_gdp_model`.

### Reformulate explicitly

Use explicit reformulation to export the transformed model or inspect all
generated values and multipliers:

```rust
model.reformulate_gdp(Hull::default())?;
let result = Highs.solve(&model, &HighsOptions::default())?;
```

To transform an independent copy instead, use `model.to_reformulated_gdp_model(...)`.

### Planning and expansion limits

Inspect a fully validated reformulation without changing the source model:

```rust
use oximo::prelude::*;

let model = Model::new("planned Hull");
variable!(model, 0.0 <= flow <= 4.0);
boolean_variable!(model, active);
disjunct_constraint!(model, active, flow <= 2.0);

let options = GdpReformulationOptions::new(Hull::default())
    .with_size_limits(GdpSizeLimits {
        variables: Some(10_000),
        constraints: Some(50_000),
        expression_nodes: Some(100_000),
    })
    .with_planning_limits(GdpPlanningLimits {
        row_expansions: Some(200_000),
        expression_nodes: Some(150_000),
    });

let summary = model.plan_gdp(options.clone())?;
println!("generated copies: {}", summary.hull_copies);
println!("final model size: {:?}", summary.total);

let report = model.reformulate_gdp(options)?;
println!("planning work: {:?}", report.planning);
```

`summary.generated` and `summary.total` give exact counts of variables,
constraints, and committed expression nodes. Totals include existing objects
and native constraints. `hull_copies` and `hull_patterns` count Hull artifacts.
Planning performs full validation and builds temporary artifacts, but leaves the
source and its dependency locks unchanged. The summary is not a reusable plan;
subsequent reformulation plans again against the then-current source model.

`GdpSizeLimits` caps the final model, including existing objects, across all groups.
`GdpPlanningLimits` additionally caps cumulative row expansions and private arena
nodes, including cloned source nodes and temporary rewrites. `summary.planning`
and `report.planning` expose `row_expansions` and `private_expression_nodes`.

Set total limits when enumerating nested groups: a small per-group pattern limit
can still produce thousands of copies and rows across ancestors. A size or
planning limit failure leaves the source unchanged, including when requesting
an independent transformed copy. `None` leaves the corresponding resource unlimited.

## Implementation details

The sections above cover choosing and applying a method. The following explains
the generated algebraic constraints, bound estimation, and domain checks for
users who want to inspect or tune the reformulation.

The API represents methods with `GdpMethod`, which includes `BigM` and `Hull`.
It is non-exhaustive, so matching it requires a wildcard arm.

### Big-M values

For a conditional row \\(Y \\Rightarrow l \\le f(x) \\le u\\), let \\(y\\) be the
binary counterpart of the Boolean decision \\(Y\\). Big-M generates the applicable
sides of:

$$
f(x) \\ge l - M\_l(1-y), \\qquad
f(x) \\le u + M\_u(1-y).
$$

At `y = 1`, these enforce the original row. At `y = 0`, sufficiently large
\\(M\\) values relax it over the global variable bounds. Equalities and ranges have
separate lower and upper \\(M\\) values. Values are nonnegative magnitudes; zero is valid.

#### Choosing \\(M\\) {#choosing-m}

To choose the best \\(M\\) value, start with the tightest **valid global bounds** you can
derive from the problem. They must cover every feasible operating mode, including
when the row's indicator is false. Tight bounds allow smaller \\(M\\) values and strengthen
the LP relaxation, where binary counterparts can take fractional values. Large \\(M\\) values
weaken that relaxation and increase differences in coefficient scale, which can
slow solving and cause numerical problems. An \\(M\\) that is too small can remove
feasible solutions.

Prefer a per-row explicit \\(M\\) when you can justify a tighter value using
problem-specific restrictions that automatic estimation does not capture.
Use a fallback only when a single justified value covers every unresolved row
side to which it applies. Neither an explicit \\(M\\) nor a fallback establishes a
missing bound or repairs a nonlinear domain. Check the \\(M\\) values and their
origins in the returned reformulation report before tuning solver settings.

#### Automatic estimation and bound tightening

Automatic estimation uses the current global variable bounds and parameter
values. By default, an affine propagation pass also tightens bounds using active
unconditional algebraic constraints connected to pending GDP expressions.
Variables and parameters used by supporting global rows are locked along with
GDP dependencies after successful reformulation. Use
`GdpReformulationOptions::default().with_bound_tightening(false)` to skip this preprocessing.

Only small affine rows are used for tightening; unsupported or unexamined rows
retain estimates from the inherited variable bounds. Nested rows additionally
use affine parent and ancestor constraints in separate conditional contexts, as
explained in [Hierarchical Big-M](#hierarchical-big-m).

Conditional rows do not establish global bounds. Ancestor rows establish local
bounds only, while sibling rows are excluded.

#### Explicit values and fallbacks

Each row side uses the first available value in the table below. Fallbacks are
consulted only when automatic estimation cannot supply a finite \\(M\\). There is no
large default \\(M\\).

| Priority | Source                                         | How to set it                                                                     |
| -------- | ---------------------------------------------- | --------------------------------------------------------------------------------- |
| 1        | Explicit value for this row side               | `options.with_big_m(row, BigMValues::upper(value))` or `BigMValues::lower(value)` |
| 2        | Automatic estimate                             | Global variable bounds and enabled bound tightening                               |
| 3        | Fallback on this disjunction's method override | `options.with_method(choice, BigM::default().with_fallback_big_m(value))`         |
| 4        | Options-level global fallback                  | `options.with_fallback_big_m(value)`                                              |
| 5        | Default-method global fallback                 | `GdpReformulationOptions::new(BigM::default().with_fallback_big_m(value))`        |

Use a default-method fallback when reusing a configured `BigM` method; use the
options-level fallback to replace its global fallback without changing the method.
If both are set, the options-level value wins.

Priorities 3 and 5 use the same `BigM::default().with_fallback_big_m(value)`
constructor; its scope depends on where you attach it. `with_method(choice, method)`
sets a fallback only for that disjunction, while `new(method)` or passing the
method directly to reformulation sets the default global fallback. An
options-level global fallback does not replace a disjunction's local fallback.

Standalone conditional rows have no disjunction fallback. Nested disjunctions
use their own override; a parent's fallback does not become a child's fallback.

Use `BigMValues::lower(value)` or `BigMValues::upper(value)` for one-sided
overrides. `options.with_range_big_m(range, values)` sets both sides of a
constant interval or maps them to the appropriate split rows when bounds are
expressions.

#### Hierarchical Big-M

For a child constraint \\(h(x) \\le 0\\) with binary indicator \\(w\\) and
parent indicator \\(p\\), the default Big-M method uses the parent's affine
constraints to estimate a local relaxation amount. It emits:

$$
h(x) \\le M\_{\\mathrm{local}}(1-w)
       + (M\_{\\mathrm{global}}-M\_{\\mathrm{local}})(1-p).
$$

When the parent is active, the child row uses the smaller local \\(M\\). When both
parent and child are inactive, the two amounts restore the global \\(M\\), so the row
remains relaxed over the global domain. For deeper nesting, ancestor contexts
inherit the bounds of their own ancestors; the differences between successive
\\(M\\) estimates give additional ancestor terms. Sibling constraints are excluded.
The differences are rounded outward to avoid reducing the inactive allowance.

This follows the hierarchical Big-M formulation in Theorem 2 and equations
(8a)-(12) of Perez and Grossmann (2024),
[Extensions to generalized disjunctive programming: hierarchical structures and first-order logic](https://link.springer.com/article/10.1007/s11081-023-09831-x).
oximo estimates \\(M\\) values using affine propagation and interval bounds without
solving the paper's auxiliary optimization problems. These estimates can match
the tightest \\(M\\) values, but may be larger when propagation cannot capture all
constraints together. Larger valid \\(M\\) values preserve integer feasibility but
can weaken the continuous relaxation and slow solving.

For example, with \\(0 \\le x \\le 10\\), a parent constraint \\(x \\le 4\\)
and child constraint \\(x \\le 1\\), the generated child row is, apart from
outward rounding:

$$
x \\le 1 + 3(1-w) + 6(1-p).
$$

At \\(p=0.5\\) and \\(w=0.25\\), this gives \\(x \\le 6.25\\); the
single-indicator child row \\(x \\le 1+9(1-w)\\) gives \\(x \\le 7.75\\).
The parent row additionally gives \\(x \\le 7\\), which is still weaker.

Hierarchical tightening is enabled by default. To compare with the
single-indicator formulation, use:

```rust
model.reformulate_gdp(BigM::default().with_hierarchical_tightening(false))?;
```

The setting also works with `with_method` for individual disjunctions. Disabling
`GdpReformulationOptions::with_bound_tightening` skips both global and hierarchical
propagation. Explicit \\(M\\) values and fallbacks retain their meaning as global
relaxation amounts. Nonlinear expressions still undergo global domain validation
before any local estimates are used.

For each row, `report.rows[i].lower_m` and `upper_m` retain the global amounts
and their origins. When a row is split, `hierarchical_m` lists the actual
nonnegative lower and upper amounts for the child and ancestor indicators.
It is empty when no side benefits from a split.

#### Missing \\(M\\): failure and fixes {#missing-m-failure-and-fixes}

The following model fails because the inactive row has no finite global upper bound
on \\(x\\). The upper-only row \\(x \\le 2\\) needs only an upper \\(M\\); the missing
lower bound does not matter:

```rust
use oximo::prelude::*;

let model = Model::new("missing M");
variable!(model, x);
boolean_variable!(model, active);
let row = disjunct_constraint!(model, active, limit, x <= 2.0);

assert!(matches!(
    model.reformulate_gdp(BigM::default()),
    Err(GdpError::MissingBigM { side: BoundSide::Upper, .. })
));
```

Choose one of the following fixes:

**Fix 1: tighter global bounds.** If the process guarantees \\(0 \\le x \\le 22\\),
encode that restriction when rebuilding the source. The upper \\(M\\) is then estimated
as 20, apart from numerical rounding:

```rust
let bounded = Model::new("bounded M");
variable!(bounded, 0.0 <= x <= 22.0);
boolean_variable!(bounded, active);
disjunct_constraint!(bounded, active, limit, x <= 2.0);
bounded.reformulate_gdp(BigM::default())?;
```

**Fix 2: a justified explicit \\(M\\).** Alternatively, the process limit can be
represented by a global constraint. On the original model, the failed call left
the source unchanged, so the following adds that restriction.

With default tightening enabled, the estimator recognizes this one-variable
affine range and estimates \\(M\\) as 20; an explicit \\(M\\) would be redundant. This
alternative deliberately disables tightening, so estimation uses the declared,
unbounded variable and the supplied \\(M\\) is needed:

```rust
constraint!(model, physical_limit, 0.0 <= x <= 22.0);
let options = GdpReformulationOptions::default()
    .with_bound_tightening(false)
    .with_big_m(row, BigMValues::upper(20.0));

model.reformulate_gdp(options)?;
```

Here \\(M = 20\\) is justified by the global limit, since the inactive row becomes
\\(x \\le 2 + 20 = 22\\). Prefer the default automatic estimate for this simple
range when tightening is enabled. An explicit \\(M\\) is also useful when a valid
limit follows from more complex restrictions that the estimator does not use.
If \\(x\\) truly has no finite upper limit when inactive, **no finite \\(M\\) is valid**
for this row.

#### Nonlinear domains

Big-M retains the original nonlinear expression even when the branch is inactive.
Declared global bounds, together with any enabled tightening from unconditional
constraints, must establish that every expression is defined.
For example, `x.ln()` needs a strictly positive global lower bound. A conditional
constraint imposing positivity inside the same branch does not establish that
global domain.

Unsafe logarithms, roots, divisions, powers, inverse functions, and tangent poles
return `GdpError::UnsafeDomain`, including when an explicit \\(M\\) is supplied.
For example, the following fails because \\(x=0\\) is permitted globally, even
though a conditional row would make \\(x\\) positive when selected:

```rust
use oximo::prelude::*;

let model = Model::new("unsafe logarithm");
variable!(model, 0.0 <= x <= 10.0);
boolean_variable!(model, active);
disjunct_constraint!(model, active, positive, x >= 1.0);
let log_row = disjunct_constraint!(model, active, log_cap, x.ln() <= 1.0);
let options = GdpReformulationOptions::default()
    .with_big_m(log_row, BigMValues::upper(10.0));

assert!(matches!(
    model.reformulate_gdp(options),
    Err(GdpError::UnsafeDomain { operation: "log", .. })
));
```

Choose one of the following fixes:

**Fix 1: a safe variable domain.** If positivity is valid for every operating
mode, rebuild with a strictly positive global lower bound:

```rust
let safe = Model::new("safe logarithm");
variable!(safe, 1.0 <= x <= 10.0);
boolean_variable!(safe, active);
disjunct_constraint!(safe, active, log_cap, x.ln() <= 1.0);
safe.reformulate_gdp(BigM::default())?;
```

**Fix 2: a global constraint that establishes the domain.** On the original
model, make the positivity restriction unconditional. The failed call applied
no reformulation or locks, and its explicit-\\(M\\) option was not stored.
The new `x >= 1.0` constraint is a small, one-variable affine global row, so
default tightening uses it to establish positivity before the domain check:

```rust
constraint!(model, global_positive, x >= 1.0);
model.reformulate_gdp(BigM::default())?;
```

If tightening is disabled, the declared lower bound remains zero for domain
analysis and this retry still returns `UnsafeDomain`.

An explicit \\(M\\) is **not** a fix for `UnsafeDomain`. The logarithm remains in the
generated algebraic expression when inactive. If nonpositive values must remain
feasible in other modes, this formulation needs a domain-safe modeling change.

### Hull transformation

The Hull transformation disaggregates variables into continuous copies scaled by
branch activations. The following describes the generated rows, nested scopes,
and nonlinear perspectives. For requirements and configuration, see [Hull](#hull).

#### Affine rows and reference coordinates

Consider a root exactly-one group with binary indicators \\(y\_i\\), a source
coordinate \\(x\\) bounded by \\(L\\) and \\(U\\), and reference \\(r\\).
For each branch, the full-copy formulation introduces the shifted copy

$$
v\_i = y\_i(x-r), \\qquad
(L-r)y\_i \\le v\_i \\le (U-r)y\_i.
$$

The aggregation equation recovers the source coordinate:

$$
x = r + \\sum\_i v\_i, \\qquad \\sum\_i y\_i = 1.
$$

For an affine conditional row \\(a^{\\mathsf T}x \\le b\\), the generated row is

$$
a^{\\mathsf T}v\_i + (a^{\\mathsf T}r-b)y\_i \\le 0.
$$

At \\(y\_i=1\\), this recovers the original inequality. At \\(y\_i=0\\), the
copy bounds force \\(v\_i=0\\). Lower bounds are transformed in the same way;
equalities and ranges generate both applicable sides. A standalone conditional
block uses an implicit on/off Hull, retaining a copy for the inactive alternative.

References default to zero. Set a coordinate with
`GdpReformulationOptions::with_hull_reference_coordinate(variable, reference)`.
A reference shifts affine copies and rows as well as nonlinear perspectives:
**a generated copy represents `activation * (source - reference)`.**
Use `with_hull_reference_coordinates(variables.iter().map(|(_, v)| (v, reference)))`
for an indexed family, or supply different values for each entry.
Setters require bare variable handles from the source model.

In flat root `ExactlyOne` groups, two or more branches that never use a coordinate
share one unused copy activated by their summed selectors. If just one branch
uses the coordinate, this activation is its complement. Otherwise a continuous
`BranchSum` weight records the unused activation. Coordinates with the same
branch support can share a weight. This preserves the projection of the full-copy
Hull relaxation while reducing generated artifacts. Nested groups retain scoped copies.

#### Nested Hull

Nested disjunctions are transformed inside-out, starting with the deepest groups.
A child's generated rows and auxiliaries remain in the enclosing branch's scope,
while parent reformulation links child copies to the appropriate parent copy.
With zero references, a parent copy \\(v\_p\\) and its exactly-one child copies
satisfy

$$
\\sum\_j v\_{pj} = v\_p, \\qquad \\sum\_j y\_{pj} = y\_p.
$$

When the parent is inactive, its copies and descendant selectors are zero.
This preserves the hierarchy described in Section 3.3 of
[Perez and Grossmann (2024)](https://link.springer.com/article/10.1007/s11081-023-09831-x).
Each nested group retains its own scope.

```rust
use oximo::prelude::*;
use oximo::{HighsOptions, solvers::Highs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("nested Hull technology selection");
    variable!(model, 0.0 <= flow <= 8.0);
    boolean_variable!(model, present);
    boolean_variable!(model, absent);
    boolean_variable!(model, technology[i in 0..2]);

    disjunct_constraint!(model, absent, flow == 0.0);
    let limits = [3.0, 8.0];
    disjunct_constraint!(model, technology[i],
        capacity[i in 0..2], flow <= limits[i]);
    disjunction!(model, [present, absent]);
    disjunction!(model, [technology[0], technology[1]], parent = present);
    objective!(model, Max, flow);

    let result = Highs
        .with_gdp(Hull::default())
        .solve(&model, &HighsOptions::default())?;
    println!("flow: {:?}", result.value_of(flow)?);
    println!("high-capacity technology: {:?}", result.boolean_value_of(technology[1])?);
    Ok(())
}
```

The optimum is `flow = 8.0`, with `present` and `technology[1]` selected.
Hull and Big-M can be mixed within this hierarchy using per-disjunction
`with_method` overrides. Big-M stages inside mixed subtrees use global bounds
for safe \\(M\\) estimation; independent subtrees using only Big-M retain the
[ancestor tightening](#hierarchical-big-m) described above.

#### Nonlinear perspectives and domains

For a nonlinear conditional row \\(f(x) \\le u\\), Hull uses the regularized
perspective below, with \\(v=y(x-r)\\):

$$
\\begin{aligned}
&d(y) = \\varepsilon + (1-\\varepsilon)y, \\\\
&d(y) f\\!\\left(\\frac{v}{d(y)}+r\\right)
     - \\varepsilon f(r)(1-y) \\le uy.
\\end{aligned}
$$

`Hull::default()` uses epsilon `1e-6`; `Hull::default().with_epsilon(value)` requires a
finite value strictly between zero and one. At binary activations the transformed
row recovers the source behavior. For convex disjunct sets, the continuous
relaxation approximates the closed perspective as epsilon approaches zero.
Nonconvex source rows remain nonconvex. Choose a backend that supports the
generated nonlinear expressions, such as SCIP.

A reference need not satisfy the branch's constraints, but it must make reference
evaluations and the generated perspective domains safe. Validation covers the
entire independent box of generated variable bounds, including points outside
the scaled bound rows. A reference inside the source domain can therefore fail:
for `flow` in `[1, 4]` in the [reference example](#reference-example), reference
`1` works, while references `2` and `4` make the
generated logarithm argument box cross zero. `UnsafeHullDomain` identifies the
operation, argument intervals, source row, scope, and whether validation failed
at the reference or over the generated box.

Large reference shifts can also lose significant precision in bounds or transformed
rows. Recovery checks account for summation and multiplication roundoff.
`HullRowPrecisionLoss` includes the absolute recovery `error` and allowed
`tolerance`; choose a closer reference or rescale the row when it occurs.
Zero effective bounds allow ordinary roundoff based on the retained source
coefficients, independently of reference magnitudes. Nonzero effective bounds
use a relative precision check.

#### Inclusive disjunctions

With [inclusive enumeration enabled](#inclusive-selection), Hull transforms
each nonempty selection pattern as an alternative.

An inclusive group with `n` branches creates `2^n - 1` nonempty patterns. Each
pattern enforces every selected branch's rows, so this retains simultaneous
selection rather than replacing it with exactly-one semantics. Pattern weights
are continuous and linked to the original binary indicators. The pattern limit
is checked before allocation, separately for each group; nested groups retain
their own scopes.

Physical coordinates retain their supplied references. Descendant selectors
and generated child auxiliaries use reference zero when copied by an inclusive
ancestor, so an inactive nested group recombines to zero. Reports record each
copy's effective reference in its scope.

## Errors and recovery

Reformulation validates domains, references, \\(M\\) values, size limits, and generated
numbers before changing the model. A reformulation error leaves the source unchanged, so you can correct
the data or options and retry.
The result-query rows also cover Boolean and Hull activation ownership.

| Error                                                | Cause                                                                                              | Fix                                                                                                                                                             |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GdpError::MissingBigM`                              | A row side has no explicit value, finite estimate, or fallback.                                    | Establish valid global bounds or supply a justified explicit \\(M\\); see [Missing \\(M\\)](#missing-m-failure-and-fixes).                                      |
| `GdpError::UnsafeDomain`                             | A nonlinear expression is undefined somewhere in the global domain, even with its branch inactive. | Establish a safe domain through global bounds or supported unconditional constraints; \\(M\\) alone cannot fix it. See [Nonlinear domains](#nonlinear-domains). |
| `GdpError::InvalidBigM`                              | An \\(M\\) value is negative, infinite, or NaN.                                                    | Supply a finite nonnegative magnitude; zero is accepted.                                                                                                        |
| `GdpError::InvalidExpression`                        | A conditional row has invalid numeric values or variable bounds.                                   | Check parameters, expression constants, and variable bounds.                                                                                                    |
| `GdpError::NumericOverflow`                          | Generated coefficients or bounds are not representable as finite numbers.                          | Rescale the model, tighten bounds, and reduce oversized \\(M\\) values while preserving validity.                                                               |
| `GdpError::PrecisionLoss`                            | Big-M shifts lose significant precision in the active row.                                         | Tighten bounds or supply a smaller valid \\(M\\); review coefficient scaling.                                                                                   |
| `GdpError::MissingHullBounds`                        | A copied coordinate lacks finite lower or upper bounds.                                            | Supply valid global bounds or derive them through unconditional affine constraints.                                                                             |
| `GdpError::InclusiveHull`                            | Hull encountered `AtLeastOne` with enumeration disabled.                                           | Enable `HullInclusivePolicy::Enumerate` or select Big-M for that group.                                                                                         |
| `GdpError::HullPatternLimit`                         | An inclusive group's patterns exceed its configured limit.                                         | Review group size and increase the limit only with acceptable expansion costs.                                                                                  |
| `GdpError::InvalidHullEpsilon`                       | Epsilon is non-finite or outside the open interval `(0, 1)`.                                       | Use a finite epsilon strictly between zero and one.                                                                                                             |
| `GdpError::InvalidHullReference`                     | A reference coordinate is infinite or NaN.                                                         | Supply a finite reference.                                                                                                                                      |
| `GdpError::ExpectedHullReferenceVariable`            | A reference setter received a compound expression or constant.                                     | Pass a bare variable handle. Foreign handles are reported separately.                                                                                           |
| `GdpError::UnsafeHullDomain`                         | A reference evaluation or generated perspective box crosses a function's domain.                   | Inspect the operation, argument intervals, scope and context; choose a safe reference or another method.                                                        |
| `GdpError::HullReferencePrecisionLoss`               | A shift loses precision in a copied coordinate's bounds.                                           | Choose a closer reference or rescale the coordinate.                                                                                                            |
| `GdpError::HullRowPrecisionLoss`                     | An active coefficient or effective bound cannot be recovered accurately.                           | Inspect `error` and `tolerance`; choose a closer reference or rescale the row.                                                                                  |
| `GdpError::SizeLimit` / `GdpError::PlanningLimit`    | Total committed size or temporary planning work exceeds a configured limit.                        | Reduce expansion or review and raise the corresponding limit.                                                                                                   |
| `GdpError::ForeignHandle`                            | Options refer to an unknown handle or a handle from another model.                                 | Use handles belonging to the model being reformulated; rebind handles for independent copies.                                                                   |
| `GdpError::Capacity`                                 | Reformulation would exceed the model's numeric ID capacity.                                        | Reduce or partition the model.                                                                                                                                  |
| `GdpSolveError::Reformulation`                       | `with_gdp(...).solve(...)` wraps a `GdpError`; the backend has not run.                            | Fix the enclosed GDP error and retry; the model is unchanged.                                                                                                   |
| `GdpSolveError::Solver`                              | The backend failed after successful reformulation.                                                 | Inspect the enclosed solver error and backend configuration. Reformulation remains applied, including its locks.                                                |
| `BooleanValueError::NonBooleanValue`                 | A selection value is non-finite or farther than `1e-5` from both zero and one.                     | Check the solution's feasibility and integrality. For an LP relaxation, query the numeric value with `value_of(handle.binary())`.                               |
| `BooleanValueError::ModelMismatch`                   | The Boolean handle belongs to another model, even if no solution value is available.               | Query with a handle from the solved model; rebind handles when using a reformulated copy.                                                                       |
| `ModelMismatchError` from `hull_activation_value_of` | The activation handle or best solution belongs to another model.                                   | Obtain a handle from the solved model's reformulation report.                                                                                                   |

## Lifecycle, cloning, and reports

`model.reformulate_gdp(options)` transforms the model in place. It validates the
complete plan before making changes; an error leaves the model unchanged.
Repeated calls transform only pending GDP components. Changing an already applied
method requires a fresh source model.

Use `model.to_reformulated_gdp_model(options)` to transform an independent copy
and retain the source for other data or method choices. Query the copy's results
using handles belonging to that copy:

```rust
let transformed = model.to_reformulated_gdp_model(BigM::default())?;
let transformed_flow = transformed.variable_handle(flow.var_id().unwrap());
let transformed_large = transformed.boolean_handle(large.id());
let result = Highs.solve(&transformed, &HighsOptions::default())?;
println!("flow: {:?}", result.value_of(transformed_flow)?);

let selected = result.boolean_value_of(transformed_large)?;
println!("large unit selected: {selected:?}");
```

Call the clone API before an in-place transformation when you want to retain
an unreformulated source. The copy enforces its own reformulation locks.

Successful transformations retain source records and provenance:

| API                               | Contents                                                                                                                                         |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `model.gdp()`                     | Borrowed view of Boolean decisions, conditional rows, disjunctions, and logical source constraints                                               |
| `model.gdp_snapshot()`            | Owned copy of the GDP source records                                                                                                             |
| `model.gdp_reformulations()`      | Transformation history                                                                                                                           |
| Returned `GdpReformulationReport` | Generated rows and auxiliaries, Boolean/binary mappings, methods, per-side \\(M\\) values, and Hull copies, references, patterns and branch sums |

### Hull copy and activation reports

`report.hull_variables` describes each copy through its `source`, `generated`,
`bounds`, effective `reference`, and activation. The `source` can be another
generated child copy, selector, or pattern weight. `constraints` lists the copy's
bound artifacts, including ancestor transformations; use `report.rows` to trace
the source's conditional rows. `hull_patterns` records inclusive selections and
weights. `hull_branch_sums` records weights shared by unused branches.

`activation_kind` distinguishes an indicator, complement, joint pattern, or branch
sum. Copies activated by complements, patterns, or branch sums have
`indicator = None`. The raw `activation` expression ID is also available.

Obtain model-bound activation handles through `report.hull_activation(copy_index)`
or `report.hull_activations()` in copy order, then query numeric values without
evaluating an expression ID:

```rust
let reports = model.gdp_reformulations();
for report in reports.iter() {
    for (copy, activation) in report.hull_variables.iter().zip(report.hull_activations()) {
        let value = result.hull_activation_value_of(activation)?;
        println!("copy {:?}, activation: {value:?}", copy.generated);
    }
}
```

This helper works on `SolverResult` and `SolutionPoint`, preserves fractional
relaxation values, and returns `Ok(None)` for missing values or solutions.
It rejects a foreign handle even without a solution, and checks the best
solution's identity for `SolverResult`. Use the transformed copy's report when
querying that copy's result. When a model is copied, its reports rebind ownership while sharing
artifact storage.

Transformed disjunct groups cannot be extended. Bound and parameter mutations
that would invalidate transformed rows are rejected. To change dependent data,
update the unreformulated source and create a fresh transformed copy.

Ordinary solver calls, persistent solves, and LP/MPS/NL export reject pending
GDP components. `with_gdp(...).solve(...)` reformulates them before invoking the
backend. Use `model.has_unreformulated_gdp()` to check whether reformulation is
still required. Exported files contain the resulting algebraic model.

## Examples and further reading

From the oximo repository, run:

```bash
cargo run -p oximo --example gdp_big_m --features gdp,highs
cargo run -p oximo --example gdp_nonlinear --features gdp,scip
cargo run -p oximo --example gdp_hull_nonlinear --features gdp,scip
```

- [Modeling](../modeling/): variables, expressions, and indexed constraints.
- [Solvers](../solvers/): choose a backend for the reformulated model.
- [Results](../results/): query algebraic variable values and solve status.
- [I/O](../io/): export the reformulated model.
