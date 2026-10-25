===description===
PHP 8.0's texts; destructuring a nullsafe chain by reference is allowed.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
function f(?null $x, $a) {
//       ^ MissingReturnType: Function f() has no return type annotation
//         ^^^^^^^^ UnusedParam: Parameter $x is never used
//          ^^^^ CompileError: Null can not be used as a standalone type [invalid-nullable-type]
//                   ^^ MissingParamType: Parameter $a of f() has no type annotation
    echo $a{0};
//       ^^^^^ MixedArrayAccess: Array access on mixed type
//         ^ CompileError: Array and string offset access syntax with curly braces is no longer supported [curly-offset-access]
    [&$y] = $a?->b;
//    ^^ MixedAssignment: Variable $y is assigned a mixed type
//    ^^ UnusedVariable: Variable $y is never read
}
function &g($a) { return $a?->b; }
//        ^ MissingReturnType: Function g() has no return type annotation
//          ^^ MissingParamType: Parameter $a of g() has no type annotation
//                       ^^^^^^ CompileError: Cannot take reference of a nullsafe chain [reference-to-nullsafe]
interface I {
    final function f();
//  ^^^^^^^^^^^^^^^^^^^ CompileError: Access type for interface method I::f() must be omitted [invalid-interface-method-modifier]
//  ^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function I::f() has no return type annotation
}
