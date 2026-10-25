===description===
Built-in attributes on a target PHP does not allow, or repeated, are compile errors from the version that defines them.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
#[Override]
//^^^^^^^^ CompileError: Attribute "Override" cannot target class (allowed targets: method) [attribute-target-mismatch]
class A {
    public function m(): void {}
}

#[ReturnTypeWillChange]
//^^^^^^^^^^^^^^^^^^^^ CompileError: Attribute "ReturnTypeWillChange" cannot target function (allowed targets: method) [attribute-target-mismatch]
function f(#[Override] int $a): void {}
//           ^^^^^^^^ CompileError: Attribute "Override" cannot target parameter (allowed targets: method) [attribute-target-mismatch]
//                     ^^^^^^ UnusedParam: Parameter $a is never used

final class B extends A {
    #[Override] #[Override]
//                ^^^^^^^^ CompileError: Attribute "Override" must not be repeated [repeated-attribute]
    public function m(): void {}
}
