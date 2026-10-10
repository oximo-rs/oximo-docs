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
constraints that a compatible solver can handle. GDP components must be
reformulated before solving or exporting. Use `reformulate_gdp` in place,
`to_reformulated_gdp_model` for an independent copy, or `with_gdp` at solve time.

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
variable \\(c\\) and use a disjunction. The symbol \\(\\bigvee\\) denotes inclusive
OR, where at least one block must be selected. Exactly-one selection is an additional
rule imposed by `disjunction!` by default.

$$
\\begin{aligned}
\\max\\quad & q-c \\\\
\\text{subject to}\\quad &
\\left[\\begin{array}{c}
Y\_{\\mathrm{small}} \\\\
c = 0
\\end{array}\\right]
\\bigvee
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

A disjunction specifies which blocks must be selected. Here the blocks cannot
both hold, they require incompatible flow ranges and different values of \\(c\\).
Their disjunction therefore selects exactly one even with inclusive OR. The formulation is:

$$
\\begin{aligned}
\\max\\quad & q-c \\\\
\\text{subject to}\\quad &
\\left[\\begin{array}{c}
Y\_{\\mathrm{small}} \\\\
q \\le 3 \\\\
c = 0
\\end{array}\\right]
\\bigvee
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

    let report = model.reformulate_gdp(BigM::default())?;
    let result = Highs.solve(&model, &HighsOptions::default())?;

    println!("{} conditional rows reformulated", report.rows.len());
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
4 conditional rows reformulated
objective: Some(6.0)
flow: Some(8.0)
cost: Some(2.0)
large unit selected: Some(true)
```

`report.rows.len()` counts the source conditional rows transformed by this call.
In this example, two in each disjunct, for a total of four. Each report entry's
`constraints` field lists its generated algebraic row IDs; a range or equality can
generate two rows.

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
native indicators, and SOS constraints are not currently supported.**

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
\\operatorname{ExactlyOne}(Y\_i : i \\in I).
$$

This is stronger than inclusive OR. Supply `AtLeastOne` for the inclusive form,
which requires

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
\\bigvee
\\left[\\begin{array}{c}
Y\_1 \\\\
q \\le 8
\\end{array}\\right]
\\end{array}\\right]
\\bigvee
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

### Hierarchical Big-M

For a child constraint \\(h(x) \\le 0\\) with binary indicator \\(w\\) and
parent indicator \\(p\\), the default Big-M method uses the parent's affine
constraints to estimate a local relaxation amount. It emits:

$$
h(x) \\le M\_{\\mathrm{local}}(1-w)
       + (M\_{\\mathrm{global}}-M\_{\\mathrm{local}})(1-p).
$$

When the parent is active, the child row uses the smaller local M. When both
parent and child are inactive, the two amounts restore the global M, so the row
remains relaxed over the global domain. For deeper nesting, ancestor contexts
inherit the bounds of their own ancestors; the differences between successive
M estimates give additional ancestor terms. Sibling constraints are excluded.
The differences are rounded outward to avoid reducing the inactive allowance.

This follows the hierarchical Big-M formulation in Theorem 2 and equations
(8a)-(12) of Perez and Grossmann (2024),
[Extensions to generalized disjunctive programming: hierarchical structures and first-order logic](https://link.springer.com/article/10.1007/s11081-023-09831-x).
oximo estimates M values using affine propagation and interval bounds without
solving the paper's auxiliary optimization problems. These estimates can match
the tightest M values, but may be larger when propagation cannot capture all
constraints together. Larger valid M values preserve integer feasibility but
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
Declaration order does not change these estimates.

Hierarchical tightening is enabled by default. To compare with the
single-indicator formulation, use:

```rust
model.reformulate_gdp(BigM::default().with_hierarchical_tightening(false))?;
```

The setting also works with `with_method` for individual disjunctions. Disabling
`GdpReformulationOptions::with_bound_tightening` skips both global and hierarchical
propagation. Explicit M values and fallbacks retain their meaning as global
relaxation amounts. Nonlinear expressions still undergo global domain validation
before any local estimates are used.

For each row, `report.rows[i].lower_m` and `upper_m` retain the global amounts
and their origins. When a row is split, `hierarchical_m` lists the actual
nonnegative lower and upper amounts for the child and ancestor indicators.
It is empty when no side benefits from a split.

## Reformulation methods

> **Keep a source model for repeated solves.** In-place reformulation locks the
> variable bounds and parameter values used by transformed rows, including
> dependencies of supporting global constraints. An applied method cannot be
> replaced on those rows. If you may re-solve with different data or compare
> methods, use `to_reformulated_gdp_model` before transforming the source:
>
> ```rust
> let transformed = model.to_reformulated_gdp_model(BigM::default())?;
> ```
>
> Update the unreformulated source between runs and create a fresh transformed
> copy for each run. Rebind handles when querying the copy's results; see
> [Lifecycle, cloning, and reports](#lifecycle-cloning-and-reports).

You can pass the method directly to reformulate the model in place:

```rust
model.reformulate_gdp(BigM::default())?;
```

Method objects carry their own settings. For example, this uses 100 only when
an automatic M estimate is unavailable:

```rust
model.reformulate_gdp(BigM::default().with_fallback_big_m(100.0))?;
```

### Select during solving

Use `with_gdp` to select a method on any solver. The wrapper reformulates pending
GDP components in place before calling the backend, using its ordinary options:

```rust
let result = Highs
    .with_gdp(BigM::default())
    .solve(&model, &HighsOptions::default())?;

println!("flow: {:?}", result.value_of(flow)?);
```

The transformation report is available through `model.gdp_reformulations()`.

### Per-disjunction configuration

For different settings or methods, construct `GdpReformulationOptions` with the
method for all disjunctions and override individual handles:

```rust
let gdp = GdpReformulationOptions::new(BigM::default())
    .with_method(choice, BigM::default().with_fallback_big_m(100.0));

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

Currently `BigM` is the only implemented reformulation method. `GdpMethod` is
non-exhaustive, so matching it requires a wildcard arm.

Reformulation processes only pending components. Selecting another method after
an in-place transformation does not replace the existing reformulation. To
compare methods, retain an unreformulated source and create independent copies
with `to_reformulated_gdp_model`.

## Big-M values

For a conditional row \\(Y \\Rightarrow l \\le f(x) \\le u\\), let \\(y\\) be the
binary counterpart of the Boolean decision \\(Y\\). Big-M generates the applicable
sides of:

$$
f(x) \\ge l - M\_l(1-y), \\qquad
f(x) \\le u + M\_u(1-y).
$$

At `y = 1`, these enforce the original row. At `y = 0`, sufficiently large
M values relax it over the global variable bounds. Equalities and ranges have
separate lower and upper M values. Values are nonnegative magnitudes; zero is valid.

### Choosing M

To choose the best M value, start with the tightest **valid global bounds** you can
derive from the problem. They must cover every feasible operating mode, including
when the row's indicator is false. Tight bounds allow smaller M values and strengthen
the LP relaxation, where binary counterparts can take fractional values. Large M values
weaken that relaxation and increase differences in coefficient scale, which can
slow solving and cause numerical problems. An M that is too small can remove
feasible solutions.

Prefer a per-row explicit M when you can justify a tighter value using
problem-specific restrictions that automatic estimation does not capture.
Use a fallback only when a single justified value covers every unresolved row
side to which it applies. Neither an explicit M nor a fallback establishes a
missing bound or repairs a nonlinear domain. Check the M values and their
origins in the returned reformulation report before tuning solver settings.

### Automatic estimation and bound tightening

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
bounds only; sibling rows are excluded.

### Explicit values and fallbacks

Each row side uses the first available value in the table below. Fallbacks are
consulted only when automatic estimation cannot supply a finite M. There is no
large default M.

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

### Missing M: failure and fixes

The following model fails because the inactive row has no finite global upper bound
on \\(x\\). The upper-only row \\(x \\le 2\\) needs only an upper M; the missing
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
encode that restriction when rebuilding the source. The upper M is then estimated
as 20, apart from numerical rounding:

```rust
let bounded = Model::new("bounded M");
variable!(bounded, 0.0 <= x <= 22.0);
boolean_variable!(bounded, active);
disjunct_constraint!(bounded, active, limit, x <= 2.0);
bounded.reformulate_gdp(BigM::default())?;
```

**Fix 2: a justified explicit M.** Alternatively, the process limit can be
represented by a global constraint. On the original model, the failed call left
the source unchanged, so the following adds that restriction.

With default tightening enabled, the estimator recognizes this one-variable
affine range and estimates M as 20; an explicit M would be redundant. This
alternative deliberately disables tightening, so estimation uses the declared,
unbounded variable and the supplied M is needed:

```rust
constraint!(model, physical_limit, 0.0 <= x <= 22.0);
let options = GdpReformulationOptions::default()
    .with_bound_tightening(false)
    .with_big_m(row, BigMValues::upper(20.0));
model.reformulate_gdp(options)?;
```

Here M = 20 is justified by the global limit, since the inactive row becomes
\\(x \\le 2 + 20 = 22\\). Prefer the default automatic estimate for this simple
range when tightening is enabled. An explicit M is also useful when a valid
limit follows from more complex restrictions that the estimator does not use.
If \\(x\\) truly has no finite upper limit when inactive, **no finite M is valid**
for this row.

### Nonlinear domains

Big-M retains the original nonlinear expression even when the branch is inactive.
Declared global bounds, together with any enabled tightening from unconditional
constraints, must establish that every expression is defined.
For example, `x.ln()` needs a strictly positive global lower bound. A conditional
constraint imposing positivity inside the same branch does not establish that
global domain.

Unsafe logarithms, roots, divisions, powers, inverse functions, and tangent poles
return `GdpError::UnsafeDomain`, including when an explicit M is supplied.
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
no reformulation or locks, and its explicit-M option was not stored.
The new `x >= 1.0` constraint is a small, one-variable affine global row, so
default tightening uses it to establish positivity before the domain check:

```rust
constraint!(model, global_positive, x >= 1.0);
model.reformulate_gdp(BigM::default())?;
```

If tightening is disabled, the declared lower bound remains zero for domain
analysis and this retry still returns `UnsafeDomain`.

An explicit M is **not** a fix for `UnsafeDomain`. The logarithm remains in the
generated algebraic expression when inactive. If nonpositive values must remain
feasible in other modes, this formulation needs a domain-safe modeling change.

## Errors and recovery

Reformulation validates domains, M values, and generated numbers before changing
the model. A reformulation error leaves the source unchanged, so you can correct
the data or options and retry.
The final two rows describe errors raised when reading Boolean selections from
solver results.

| Error                                | Cause                                                                                              | Fix                                                                                                                                                       |
| ------------------------------------ | -------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GdpError::MissingBigM`              | A row side has no explicit value, finite estimate, or fallback.                                    | Establish valid global bounds or supply a justified explicit M; see [Missing M](#missing-m-failure-and-fixes).                                            |
| `GdpError::UnsafeDomain`             | A nonlinear expression is undefined somewhere in the global domain, even with its branch inactive. | Establish a safe domain through global bounds or supported unconditional constraints; M alone cannot fix it. See [Nonlinear domains](#nonlinear-domains). |
| `GdpError::InvalidBigM`              | An M value is negative, infinite, or NaN.                                                          | Supply a finite nonnegative magnitude; zero is accepted.                                                                                                  |
| `GdpError::InvalidExpression`        | A conditional row has invalid numeric values or variable bounds.                                   | Check parameters, expression constants, and variable bounds.                                                                                              |
| `GdpError::NumericOverflow`          | Generated coefficients or bounds are not representable as finite numbers.                          | Rescale the model, tighten bounds, and reduce oversized M values while preserving validity.                                                               |
| `GdpError::PrecisionLoss`            | Big-M shifts lose significant precision in the active row.                                         | Tighten bounds or supply a smaller valid M; review coefficient scaling.                                                                                   |
| `GdpError::ForeignHandle`            | Options refer to an unknown handle or a handle from another model.                                 | Use handles belonging to the model being reformulated; rebind handles for independent copies.                                                             |
| `GdpError::Capacity`                 | Reformulation would exceed the model's numeric ID capacity.                                        | Reduce or partition the model.                                                                                                                            |
| `GdpSolveError::Reformulation`       | `with_gdp(...).solve(...)` wraps a `GdpError`; the backend has not run.                            | Fix the enclosed GDP error and retry; the model is unchanged.                                                                                             |
| `GdpSolveError::Solver`              | The backend failed after successful reformulation.                                                 | Inspect the enclosed solver error and backend configuration. Reformulation remains applied, including its locks.                                          |
| `BooleanValueError::NonBooleanValue` | A selection value is non-finite or farther than `1e-5` from both zero and one.                     | Check the solution's feasibility and integrality. For an LP relaxation, query the numeric value with `value_of(handle.binary())`.                         |
| `BooleanValueError::ModelMismatch`   | The Boolean handle belongs to another model, even if no solution value is available.               | Query with a handle from the solved model; rebind handles when using a reformulated copy.                                                                 |

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

| API                               | Contents                                                                                                       |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `model.gdp()`                     | Borrowed view of Boolean decisions, conditional rows, disjunctions, and logical source constraints             |
| `model.gdp_snapshot()`            | Owned copy of the GDP source records                                                                           |
| `model.gdp_reformulations()`      | Transformation history                                                                                         |
| Returned `GdpReformulationReport` | Generated row IDs, auxiliary variable IDs, Boolean/binary mappings, methods, and per-side M values and origins |

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
```

- [Modeling](../modeling/): variables, expressions, and indexed constraints.
- [Solvers](../solvers/): choose a backend for the reformulated model.
- [Results](../results/): query algebraic variable values and solve status.
- [I/O](../io/): export the reformulated model.
