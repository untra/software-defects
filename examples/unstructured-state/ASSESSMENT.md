# Unstructured state: susceptibility and corpus coverage

Susceptibility scores are qualitative editorial judgments from 1 (fewer common mechanisms) to 5 (many common mechanisms). They are not measured prevalence, detector accuracy, or example quotas. JavaScript is assessed separately from TypeScript even where the scores coincide.

## Susceptibility

| Defect | Go | Rust | JavaScript | TypeScript | Python |
|---|---:|---:|---:|---:|---:|
| [divide-by-zero](divide-by-zero/) | 4 | 3 | 5 | 5 | 4 |
| [type-errors](type-errors/) | 2 | 1 | 5 | 5 | 5 |
| [index-out-of-bounds](index-out-of-bounds/) | 4 | 3 | 5 | 5 | 4 |
| [lack-of-input-validation](lack-of-input-validation/) | 4 | 3 | 5 | 5 | 5 |
| [unconstrained-inputs](unconstrained-inputs/) | 4 | 3 | 4 | 4 | 4 |
| [duck-typing-without-contracts](duck-typing-without-contracts/) | 2 | 2 | 5 | 5 | 5 |
| [missing-undefined-returns](missing-undefined-returns/) | 3 | 2 | 5 | 5 | 5 |
| [version-library-mismanagement](version-library-mismanagement/) | 2 | 3 | 4 | 4 | 3 |
| [insecure-deserialization](insecure-deserialization/) | 3 | 2 | 4 | 4 | 4 |

## Example counts

Counts below are original standalone source files, independent of susceptibility scores. Fixed coverage is audited separately; a source file does not establish production detector support.

| Defect | Go | Rust | JavaScript | TypeScript | Python |
|---|---:|---:|---:|---:|---:|
| divide-by-zero | 4 | 3 | 5 | 5 | 4 |
| type-errors | 2 | 1 | 5 | 5 | 5 |
| index-out-of-bounds | 4 | 3 | 5 | 5 | 4 |
| lack-of-input-validation | 5 | 4 | 6 | 6 | 6 |
| unconstrained-inputs | 7 | 6 | 7 | 7 | 7 |
| duck-typing-without-contracts | 3 | 3 | 6 | 6 | 6 |
| missing-undefined-returns | 4 | 2 | 5 | 5 | 5 |
| version-library-mismanagement | 2 | 3 | 4 | 4 | 3 |
| insecure-deserialization | 3 | 2 | 4 | 4 | 4 |
| **Total** | 34 | 27 | 47 | 47 | 44 |

## Audited input and output contracts

- **lack-of-input-validation:** a boundary value reaches an operation without establishing its required shape or domain. Typed parameters can still accept invalid quantities. Dynamic JSON requires runtime checks; annotations alone do not establish them. All five languages now include a function-boundary quantity pair.
- **unconstrained-inputs:** accepted values exceed a size, work, depth, or representable-value budget. All five languages include batch cardinality, requested allocation, and multiplicative expansion pairs. Independent limits on two inputs do not establish a combined output limit.
- **duck-typing-without-contracts:** a value is asserted, downcast, or consumed as a shape/capability/unit that has not been established. Go interfaces and Rust traits prevent many structural errors, but unchecked dynamic conversions and semantic units remain relevant. JavaScript, TypeScript, and Python have additional runtime shape gaps.
- **missing-undefined-returns:** the producer omits the required output or returns a success-shaped value without the promised payload. A caller failing to check a legitimate optional result is a separate consumer defect. Intentional optional returns and command functions returning no value are not failures. Go and Rust reject ordinary missing typed returns; their examples instead model false success or omitted failure signaling.

Function labels describe boundary requirements or producer obligations. Variable labels describe values and aliases carrying those unresolved requirements. Neither label alone proves a defect or a CWE. A fixed example can retain the same syntax signal.

The four audited groups have mirrored fixes in all five languages. Production coverage must be read from the rule catalog and query tests rather than inferred from this table. See [the review](REVIEW.md) for audit findings, research, and analysis limits.

## Other mechanisms

- **divide-by-zero:** integer division and remainder can fail; JavaScript/TypeScript and floating-point operations can instead produce non-finite values. See the [division review](divide-by-zero/REVIEW.md) for the established analysis.
- **type-errors:** static types reduce ordinary mismatches in Go/Rust; dynamic values, formatting, coercion, and semantic units remain relevant. Python and JavaScript have more implicit runtime behavior; TypeScript inherits JavaScript runtime behavior.
- **index-out-of-bounds:** language differences include panics/exceptions, undefined elements, missing-map defaults, and string byte/code-unit boundaries.
- **version-library-mismanagement:** examples demonstrate incompatible copies, version selection, or features without network installation.
- **insecure-deserialization:** examples demonstrate untrusted fields modifying application state; presence of a parser alone does not prove unsafe deserialization.
