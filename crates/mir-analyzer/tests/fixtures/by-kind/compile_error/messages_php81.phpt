===description===
PHP 8.1 suggests a backing type for an int or string case value.
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum E {
    case A = 1;
//         ^ CompileError: Case A of non-backed enum E must not have a value, try adding ": int" to the enum declaration [pure-enum-case-value]
    case B = 'b';
//         ^ CompileError: Case B of non-backed enum E must not have a value, try adding ": string" to the enum declaration [pure-enum-case-value]
    case C = 1.5;
//         ^ CompileError: Case C of non-backed enum E must not have a value [pure-enum-case-value]
}
class A {
    readonly function f() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use 'readonly' as method modifier [modifier-not-allowed]
}
enum Status {
    public function __get($name) {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Enum may not include __get [enum-magic-method]
}
