---
title: Error codes
description: All MIR error codes in numeric order, with links to per-issue pages.
sidebar:
  order: 1
---

Every issue mir emits carries a stable identifier of the form `MIR####`.
The code appears in CLI output and JSON/SARIF output:

```
src/x.php:12:4 error[MIR0005] UndefinedClass: Class Foo not found
```

Codes are organized into bands by category. Bands have headroom for growth;
once a code ships, it is **never reused** for a different issue. Renamed
issues keep their code.

## Bands

| Range         | Category                  |
|---------------|---------------------------|
| MIR0001–0099  | Undefined symbols         |
| MIR0100–0199  | Nullability               |
| MIR0200–0299  | Type mismatches           |
| MIR0300–0399  | Array / offset            |
| MIR0400–0499  | Redundancy                |
| MIR0500–0599  | Dead code                 |
| MIR0600–0699  | Readonly                  |
| MIR0700–0799  | Inheritance               |
| MIR0800–0899  | Security (taint)          |
| MIR0900–0999  | Generics                  |
| MIR1000–1099  | Deprecation / internal    |
| MIR1100–1199  | Missing types / docblocks |
| MIR1200–1299  | Mixed                     |
| MIR1300–1399  | Trait                     |
| MIR1400–1499  | Parse                     |
| MIR1500–1599  | Other                     |
| MIR1600–1699  | Attributes                |

## All codes

| Code     | Name                          | Page |
|----------|-------------------------------|------|
| MIR0001  | InvalidScope                  | [undefined/invalid-scope](../undefined/invalid-scope/) |
| MIR0002  | UndefinedVariable             | [undefined/undefined-variable](../undefined/undefined-variable/) |
| MIR0003  | UndefinedFunction             | [undefined/undefined-function](../undefined/undefined-function/) |
| MIR0004  | UndefinedMethod               | [undefined/undefined-method](../undefined/undefined-method/) |
| MIR0005  | UndefinedClass                | [undefined/undefined-class](../undefined/undefined-class/) |
| MIR0006  | UndefinedProperty             | [undefined/undefined-property](../undefined/undefined-property/) |
| MIR0007  | UndefinedConstant             | [undefined/undefined-constant](../undefined/undefined-constant/) |
| MIR0008  | PossiblyUndefinedVariable     | [undefined/possibly-undefined-variable](../undefined/possibly-undefined-variable/) |
| MIR0009  | UndefinedTrait                | [undefined/undefined-trait](../undefined/undefined-trait/) |
| MIR0010  | ParentNotFound                | [undefined/parent-not-found](../undefined/parent-not-found/) |
| MIR0011  | InaccessibleClassConstant     | [undefined/inaccessible-class-constant](../undefined/inaccessible-class-constant/) |
| MIR0100  | NullArgument                  | [nullability/null-argument](../nullability/null-argument/) |
| MIR0101  | NullPropertyFetch             | [nullability/null-property-fetch](../nullability/null-property-fetch/) |
| MIR0102  | NullMethodCall                | [nullability/null-method-call](../nullability/null-method-call/) |
| MIR0103  | NullArrayAccess               | [nullability/null-array-access](../nullability/null-array-access/) |
| MIR0104  | PossiblyNullArgument          | [nullability/possibly-null-argument](../nullability/possibly-null-argument/) |
| MIR0105  | PossiblyInvalidArgument       | [nullability/possibly-invalid-argument](../nullability/possibly-invalid-argument/) |
| MIR0106  | PossiblyNullPropertyFetch     | [nullability/possibly-null-property-fetch](../nullability/possibly-null-property-fetch/) |
| MIR0107  | PossiblyNullMethodCall        | [nullability/possibly-null-method-call](../nullability/possibly-null-method-call/) |
| MIR0108  | PossiblyNullArrayAccess       | [nullability/possibly-null-array-access](../nullability/possibly-null-array-access/) |
| MIR0109  | NullableReturnStatement       | [nullability/nullable-return-statement](../nullability/nullable-return-statement/) |
| MIR0200  | InvalidReturnType             | [type-mismatches/invalid-return-type](../type-mismatches/invalid-return-type/) |
| MIR0201  | InvalidArgument               | [type-mismatches/invalid-argument](../type-mismatches/invalid-argument/) |
| MIR0202  | TooFewArguments               | [type-mismatches/too-few-arguments](../type-mismatches/too-few-arguments/) |
| MIR0203  | TooManyArguments              | [type-mismatches/too-many-arguments](../type-mismatches/too-many-arguments/) |
| MIR0204  | InvalidNamedArgument          | [type-mismatches/invalid-named-argument](../type-mismatches/invalid-named-argument/) |
| MIR0205  | InvalidPassByReference        | [type-mismatches/invalid-pass-by-reference](../type-mismatches/invalid-pass-by-reference/) |
| MIR0206  | InvalidPropertyAssignment     | [type-mismatches/invalid-property-assignment](../type-mismatches/invalid-property-assignment/) |
| MIR0207  | InvalidCast                   | [type-mismatches/invalid-cast](../type-mismatches/invalid-cast/) |
| MIR0208  | InvalidOperand                | [type-mismatches/invalid-operand](../type-mismatches/invalid-operand/) |
| MIR0209  | MismatchingDocblockReturnType | [type-mismatches/mismatching-docblock-return-type](../type-mismatches/mismatching-docblock-return-type/) |
| MIR0210  | MismatchingDocblockParamType  | [type-mismatches/mismatching-docblock-param-type](../type-mismatches/mismatching-docblock-param-type/) |
| MIR0211  | InvalidStringClass            | [type-mismatches/invalid-string-class](../type-mismatches/invalid-string-class/) |
| MIR0212  | TypeCheckMismatch             | [type-mismatches/type-check-mismatch](../type-mismatches/type-check-mismatch/) |
| MIR0213  | PossiblyInvalidOperand        | [type-mismatches/possibly-invalid-operand](../type-mismatches/possibly-invalid-operand/) |
| MIR0214  | PossiblyNullOperand           | [type-mismatches/possibly-null-operand](../type-mismatches/possibly-null-operand/) |
| MIR0215  | InvalidStaticInvocation       | [type-mismatches/invalid-static-invocation](../type-mismatches/invalid-static-invocation/) |
| MIR0216  | NonStaticSelfCall             | [type-mismatches/non-static-self-call](../type-mismatches/non-static-self-call/) |
| MIR0217  | DirectConstructorCall         | [type-mismatches/direct-constructor-call](../type-mismatches/direct-constructor-call/) |
| MIR0218  | InvalidPropertyFetch          | [type-mismatches/invalid-property-fetch](../type-mismatches/invalid-property-fetch/) |
| MIR0219  | InvalidArrayAccess            | [type-mismatches/invalid-array-access](../type-mismatches/invalid-array-access/) |
| MIR0220  | InvalidArrayAssignment        | [type-mismatches/invalid-array-assignment](../type-mismatches/invalid-array-assignment/) |
| MIR0221  | Trace                         | [type-mismatches/trace](../type-mismatches/trace/) |
| MIR0222  | RawObjectIteration            | [type-mismatches/raw-object-iteration](../type-mismatches/raw-object-iteration/) |
| MIR0223  | PossiblyRawObjectIteration    | [type-mismatches/possibly-raw-object-iteration](../type-mismatches/possibly-raw-object-iteration/) |
| MIR0224  | InvalidNamedArguments         | [type-mismatches/invalid-named-arguments](../type-mismatches/invalid-named-arguments/) |
| MIR0225  | ArgumentTypeCoercion          | [type-mismatches/argument-type-coercion](../type-mismatches/argument-type-coercion/) |
| MIR0226  | PropertyTypeCoercion          | [type-mismatches/property-type-coercion](../type-mismatches/property-type-coercion/) |
| MIR0227  | PossiblyInvalidArrayAccess    | [type-mismatches/possibly-invalid-array-access](../type-mismatches/possibly-invalid-array-access/) |
| MIR0300  | InvalidArrayOffset            | [array/invalid-array-offset](../array/invalid-array-offset/) |
| MIR0301  | NonExistentArrayOffset        | [array/non-existent-array-offset](../array/non-existent-array-offset/) |
| MIR0302  | PossiblyInvalidArrayOffset    | [array/possibly-invalid-array-offset](../array/possibly-invalid-array-offset/) |
| MIR0400  | RedundantCondition            | [redundancy/redundant-condition](../redundancy/redundant-condition/) |
| MIR0401  | RedundantCast                 | [redundancy/redundant-cast](../redundancy/redundant-cast/) |
| MIR0402  | UnnecessaryVarAnnotation      | [redundancy/unnecessary-var-annotation](../redundancy/unnecessary-var-annotation/) |
| MIR0403  | TypeDoesNotContainType        | [redundancy/type-does-not-contain-type](../redundancy/type-does-not-contain-type/) |
| MIR0404  | ParadoxicalCondition          | [redundancy/paradoxical-condition](../redundancy/paradoxical-condition/) |
| MIR0405  | UnhandledMatchCondition       | [redundancy/unhandled-match-condition](../redundancy/unhandled-match-condition/) |
| MIR0406  | DocblockTypeContradiction     | [redundancy/docblock-type-contradiction](../redundancy/docblock-type-contradiction/) |
| MIR0407  | UnevaluatedCode               | [redundancy/unevaluated-code](../redundancy/unevaluated-code/) |
| MIR0500  | UnusedVariable                | [dead-code/unused-variable](../dead-code/unused-variable/) |
| MIR0501  | UnusedParam                   | [dead-code/unused-param](../dead-code/unused-param/) |
| MIR0502  | UnreachableCode               | [dead-code/unreachable-code](../dead-code/unreachable-code/) |
| MIR0503  | UnusedMethod                  | [dead-code/unused-method](../dead-code/unused-method/) |
| MIR0504  | UnusedProperty                | [dead-code/unused-property](../dead-code/unused-property/) |
| MIR0505  | UnusedFunction                | [dead-code/unused-function](../dead-code/unused-function/) |
| MIR0506  | UnusedForeachValue            | [dead-code/unused-foreach-value](../dead-code/unused-foreach-value/) |
| MIR0507  | UnusedClass                   | [dead-code/unused-class](../dead-code/unused-class/) |
| MIR0508  | UnusedSuppress                | [dead-code/unused-suppress](../dead-code/unused-suppress/) |
| MIR0600  | ReadonlyPropertyAssignment    | [other/readonly-property-assignment](../other/readonly-property-assignment/) |
| MIR0700  | UnimplementedAbstractMethod   | [inheritance/unimplemented-abstract-method](../inheritance/unimplemented-abstract-method/) |
| MIR0701  | UnimplementedInterfaceMethod  | [inheritance/unimplemented-interface-method](../inheritance/unimplemented-interface-method/) |
| MIR0702  | MethodSignatureMismatch       | [inheritance/method-signature-mismatch](../inheritance/method-signature-mismatch/) |
| MIR0703  | OverriddenMethodAccess        | [inheritance/overridden-method-access](../inheritance/overridden-method-access/) |
| MIR0704  | InvalidExtendClass            | [inheritance/invalid-extend-class](../inheritance/invalid-extend-class/) |
| MIR0705  | FinalMethodOverridden         | [inheritance/final-method-overridden](../inheritance/final-method-overridden/) |
| MIR0706  | AbstractInstantiation         | [inheritance/abstract-instantiation](../inheritance/abstract-instantiation/) |
| MIR0707  | CircularInheritance           | [inheritance/circular-inheritance](../inheritance/circular-inheritance/) |
| MIR0708  | InvalidOverride               | [inheritance/invalid-override](../inheritance/invalid-override/) |
| MIR0709  | InterfaceInstantiation        | [inheritance/interface-instantiation](../inheritance/interface-instantiation/) |
| MIR0710  | OverriddenPropertyAccess      | [inheritance/overridden-property-access](../inheritance/overridden-property-access/) |
| MIR0711  | AbstractMethodCall            | [inheritance/abstract-method-call](../inheritance/abstract-method-call/) |
| MIR0800  | TaintedInput                  | [security/tainted-input](../security/tainted-input/) |
| MIR0801  | TaintedHtml                   | [security/tainted-html](../security/tainted-html/) |
| MIR0802  | TaintedSql                    | [security/tainted-sql](../security/tainted-sql/) |
| MIR0803  | TaintedShell                  | [security/tainted-shell](../security/tainted-shell/) |
| MIR0804  | TaintedLlmPrompt              | [security/tainted-llm-prompt](../security/tainted-llm-prompt/) |
| MIR0900  | InvalidTemplateParam          | [generics/invalid-template-param](../generics/invalid-template-param/) |
| MIR0901  | ShadowedTemplateParam         | [generics/shadowed-template-param](../generics/shadowed-template-param/) |
| MIR0902  | IfThisIsMismatch              | [generics/if-this-is-mismatch](../generics/if-this-is-mismatch/) |
| MIR1000  | DeprecatedCall                | [other/deprecated-call](../other/deprecated-call/) |
| MIR1001  | DeprecatedMethodCall          | [other/deprecated-method-call](../other/deprecated-method-call/) |
| MIR1002  | DeprecatedMethod              | [other/deprecated-method](../other/deprecated-method/) |
| MIR1003  | DeprecatedClass               | [other/deprecated-class](../other/deprecated-class/) |
| MIR1004  | InternalMethod                | [other/internal-method](../other/internal-method/) |
| MIR1005  | DeprecatedProperty            | [other/deprecated-property](../other/deprecated-property/) |
| MIR1006  | DeprecatedInterface           | [other/deprecated-interface](../other/deprecated-interface/) |
| MIR1007  | DeprecatedTrait               | [other/deprecated-trait](../other/deprecated-trait/) |
| MIR1008  | DeprecatedConstant            | [other/deprecated-constant](../other/deprecated-constant/) |
| MIR1009  | WrongCaseFunction             | [other/wrong-case-function](../other/wrong-case-function/) |
| MIR1010  | WrongCaseMethod               | [other/wrong-case-method](../other/wrong-case-method/) |
| MIR1011  | WrongCaseClass                | [other/wrong-case-class](../other/wrong-case-class/) |
| MIR1100  | MissingReturnType             | [other/missing-return-type](../other/missing-return-type/) |
| MIR1101  | MissingParamType              | [other/missing-param-type](../other/missing-param-type/) |
| MIR1102  | MissingThrowsDocblock         | [other/missing-throws-docblock](../other/missing-throws-docblock/) |
| MIR1103  | InvalidDocblock               | [other/invalid-docblock](../other/invalid-docblock/) |
| MIR1104  | MissingPropertyType           | [other/missing-property-type](../other/missing-property-type/) |
| MIR1105  | MissingClosureReturnType      | [other/missing-closure-return-type](../other/missing-closure-return-type/) |
| MIR1200  | MixedArgument                 | [other/mixed-argument](../other/mixed-argument/) |
| MIR1201  | MixedAssignment               | [other/mixed-assignment](../other/mixed-assignment/) |
| MIR1202  | MixedMethodCall               | [other/mixed-method-call](../other/mixed-method-call/) |
| MIR1203  | MixedPropertyFetch            | [other/mixed-property-fetch](../other/mixed-property-fetch/) |
| MIR1204  | MixedClone                    | [other/mixed-clone](../other/mixed-clone/) |
| MIR1205  | InvalidClone                  | [other/invalid-clone](../other/invalid-clone/) |
| MIR1206  | PossiblyInvalidClone          | [other/possibly-invalid-clone](../other/possibly-invalid-clone/) |
| MIR1207  | InvalidToString               | [other/invalid-to-string](../other/invalid-to-string/) |
| MIR1208  | MixedPropertyAssignment       | [other/mixed-property-assignment](../other/mixed-property-assignment/) |
| MIR1209  | MixedArrayAccess              | [other/mixed-array-access](../other/mixed-array-access/) |
| MIR1210  | MixedArrayOffset              | [other/mixed-array-offset](../other/mixed-array-offset/) |
| MIR1211  | MixedFunctionCall             | [other/mixed-function-call](../other/mixed-function-call/) |
| MIR1212  | MixedReturnStatement          | [other/mixed-return-statement](../other/mixed-return-statement/) |
| MIR1300  | InvalidTraitUse               | [other/invalid-trait-use](../other/invalid-trait-use/) |
| MIR1301  | ForbiddenCode                 | [other/forbidden-code](../other/forbidden-code/) |
| MIR1400  | ParseError                    | [other/parse-error](../other/parse-error/) |
| MIR1500  | InvalidThrow                  | [other/invalid-throw](../other/invalid-throw/) |
| MIR1501  | ImplicitToStringCast          | [other/implicit-to-string-cast](../other/implicit-to-string-cast/) |
| MIR1502  | ImplicitFloatToIntCast        | [other/implicit-float-to-int-cast](../other/implicit-float-to-int-cast/) |
| MIR1503  | InvalidCatch                  | [other/invalid-catch](../other/invalid-catch/) |
| MIR1504  | NoInterfaceProperties         | [other/no-interface-properties](../other/no-interface-properties/) |
| MIR1505  | UndefinedDocblockClass        | [other/undefined-docblock-class](../other/undefined-docblock-class/) |
| MIR1506  | UnsupportedReferenceUsage     | [other/unsupported-reference-usage](../other/unsupported-reference-usage/) |
| MIR1507  | MissingConstructor            | [other/missing-constructor](../other/missing-constructor/) |
| MIR1600  | InvalidAttribute              | [attributes/invalid-attribute](../attributes/invalid-attribute/) |
| MIR1601  | UndefinedAttributeClass       | [attributes/undefined-attribute-class](../attributes/undefined-attribute-class/) |
| MIR1602  | DuplicateClass                | [attributes/duplicate-class](../attributes/duplicate-class/) |
| MIR1603  | DuplicateInterface            | [attributes/duplicate-interface](../attributes/duplicate-interface/) |
| MIR1604  | DuplicateTrait                | [attributes/duplicate-trait](../attributes/duplicate-trait/) |
| MIR1605  | DuplicateEnum                 | [attributes/duplicate-enum](../attributes/duplicate-enum/) |
| MIR1606  | DuplicateFunction             | [attributes/duplicate-function](../attributes/duplicate-function/) |
| MIR1700  | ImpurePropertyAssignment      | [other/impure-property-assignment](../other/impure-property-assignment/) |
| MIR1701  | ImpureMethodCall              | [other/impure-method-call](../other/impure-method-call/) |
| MIR1702  | ImpureGlobalVariable          | [other/impure-global-variable](../other/impure-global-variable/) |
| MIR1703  | ImpureStaticVariable          | [other/impure-static-variable](../other/impure-static-variable/) |
| MIR1704  | ImpureFunctionCall            | [other/impure-function-call](../other/impure-function-call/) |
