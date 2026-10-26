===description===
PHP 8.2's texts for modifier, reserved name and type errors; parent::$x::get() is a static call before 8.4.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
abstract class A {
    abstract final function f();
//  ^^^^^^^^^^^^^^ CompileError: Cannot use the final modifier on an abstract class member [abstract-final-conflict]
    static const C = 1;
//         ^^^^^ CompileError: Cannot use 'static' as constant modifier [invalid-constant-modifier]
    readonly function g() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use 'readonly' as method modifier [modifier-not-allowed]
    abstract public $p;
//  ^^^^^^^^^^^^^^^^^^ CompileError: Properties cannot be declared abstract [abstract-property-without-abstract-hook]
//  ^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property A::$p has no type annotation
}
interface I {
    public $p;
//  ^^^^^^^^^ CompileError: Interfaces may not include properties [interface-property-without-hooks]
}
trait T { function f() {} }
class B extends self {
//              ^^^^ CompileError: Cannot use 'self' as class name, as it is reserved [reserved-class-reference]
//              ^^^^ UndefinedClass: Class self does not exist
    use T { f as static h; }
//               ^^^^^^ CompileError: Cannot use 'static' as method modifier [invalid-trait-alias-modifier]
}
class int {}
//    ^^^ CompileError: Cannot use 'int' as class name as it is reserved [reserved-class-name]
enum string {}
//   ^^^^^^ CompileError: Cannot use 'string' as class name as it is reserved [reserved-enum-name]
function f(true|false $x, A\int $y): string {
//         ^^^^^^^^^^ CompileError: Type contains both true and false, bool should be used instead [redundant-type]
//         ^^^^^^^^^^^^^ UnusedParam: Parameter $x is never used
//                        ^^^^^ CompileError: Cannot use 'A\int' as class name as it is reserved [reserved-type-name]
//                        ^^^^^ UndefinedClass: Class A\int does not exist
//                        ^^^^^^^^ UnusedParam: Parameter $y is never used
    return (null)::class;
//         ^^^^^^ CompileError: Cannot use "::class" on value of type null [class-fetch-on-literal]
}
#[AllowDynamicProperties]
//^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot apply #[AllowDynamicProperties] to interface [allow-dynamic-properties-on-invalid-class]
interface J {}
#[AllowDynamicProperties]
//^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot apply #[AllowDynamicProperties] to readonly class R [allow-dynamic-properties-on-invalid-class]
readonly class R {}
#[AllowDynamicProperties]
enum E {}
class D { public static $x; }
//        ^^^^^^^^^^^^^^^^ MissingPropertyType: Property D::$x has no type annotation
class C extends D { function f() { parent::$x::get(); } }
enum F {
    final abstract function f();
//  ^^^^^^^^^^^^^^ CompileError: Cannot use the final modifier on an abstract class member [abstract-enum-method]
}
enum G {
    abstract function a();
//  ^^^^^^^^ CompileError: Enum G must implement 2 abstract private methods (G::a, G::b) [abstract-enum-method]
    abstract protected function b();
}
$h = new class {
//<^^ UnusedVariable: Variable $h is never read
    abstract function a();
//  ^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class class@anonymous contains 4 abstract methods and must therefore be declared abstract or implement the remaining methods (class@anonymous::a, class@anonymous::b, class@anonymous::c, ...) [abstract-anonymous-class-method]
    abstract function b();
    abstract function c();
    abstract function d();
};
