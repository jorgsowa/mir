===description===
PHP 7.4's texts; the magic method type checks PHP 8.0 added are not reported.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
class A {
    function __construct(): void {}
//                          ^^^^ CompileError: Constructor A::__construct() cannot declare a return type [magic-method-return-type]
    function __destruct(): void {}
//                         ^^^^ CompileError: Destructor A::__destruct() cannot declare a return type [magic-method-return-type]
    function __clone(): int {}
//                      ^^^ CompileError: Clone method A::__clone() cannot declare a return type [magic-method-return-type]
//                      ^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
    function __isset($n): int { return 1; }
    function __get(int $n) {}
}
interface I {
    private function f();
//  ^^^^^^^^^^^^^^^^^^^^^ CompileError: Access type for interface method I::f() must be omitted [invalid-interface-method-modifier]
//  ^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function I::f() has no return type annotation
}
function g(): string { yield 1; }
//            ^^^^^^ CompileError: Generators may only declare a return type of Generator, Iterator, Traversable, or iterable, string is not permitted [generator-return-type]
function h(): ?void {}
//             ^^^^ CompileError: Void type cannot be nullable [standalone-type-in-composite]
function k() {
//       ^ MissingReturnType: Function k() has no return type annotation
    [&$a] = 1;
//    ^^ MixedAssignment: Variable $a is assigned a mixed type
//    ^^ UnusedVariable: Variable $a is never read
//          ^ CompileError: Cannot assign reference to non referencable value [reference-to-non-referenceable]
}
class B extends self {}
//              ^^^^ CompileError: Cannot use 'self' as class name as it is reserved [reserved-class-reference]
//              ^^^^ UndefinedClass: Class self does not exist
trait Waves { function hello() {} }
trait Greets { function hello() {} }
class Collides {
    use Waves, Greets;
//             ^^^^^^^ CompileError: Trait method hello has not been applied, because there are collisions with other trait methods on Collides [trait-method-collision]
}
function class_fetch_on_array(): string { return [1]::class; }
//                                               ^^^ CompileError: Cannot use ::class with dynamic class name [dynamic-class-fetch]
