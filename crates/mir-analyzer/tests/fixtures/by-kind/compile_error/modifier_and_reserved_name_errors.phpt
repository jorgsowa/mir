===description===
Modifier conflicts and reserved class-like names are compile errors, as in PHP.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
class Modifiers {
    public private $a;
//  ^^^^^^^^^^^^^^ CompileError: Multiple access type modifiers are not allowed [multiple-visibility-modifiers]
//  ^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Modifiers::$a has no type annotation
    static static $b;
//  ^^^^^^ CompileError: Multiple static modifiers are not allowed [duplicate-modifier]
//  ^^^^^^^^^^^^^^^^ MissingPropertyType: Property Modifiers::$b has no type annotation
    static const C = 1;
//         ^^^^^ CompileError: Cannot use the static modifier on a class constant [invalid-constant-modifier]
    readonly function f() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use the readonly modifier on a method [modifier-not-allowed]
}
abstract final class Both {}
//<^^^^^^^^^^^^^^ CompileError: Cannot use the final modifier on an abstract class [abstract-final-conflict]
interface iterable {}
//        ^^^^^^^^ CompileError: Cannot use "iterable" as an interface name as it is reserved [keyword-as-declaration-name]
trait self {}
//    ^^^^ CompileError: Cannot use "self" as a trait name as it is reserved [keyword-as-declaration-name]
class int {}
//    ^^^ CompileError: Cannot use "int" as a class name as it is reserved [reserved-class-name]
function g() {
//       ^ MissingReturnType: Function g() has no return type annotation
    [&$x] = 1;
//    ^^ MixedAssignment: Variable $x is assigned a mixed type
//    ^^ UnusedVariable: Variable $x is never read
//          ^ CompileError: Cannot assign reference to non referenceable value [reference-to-non-referenceable]
    __halt_compiler();
//  ^^^^^^^^^^^^^^^ CompileError: __HALT_COMPILER() can only be used from the outermost scope [outermost-scope-only]
}
