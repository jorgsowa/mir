===description===
PHP 8.4's texts: #[\Deprecated] targets and the plural "remaining methods".
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
#[Deprecated]
//^^^^^^^^^^ CompileError: Attribute "Deprecated" cannot target class (allowed targets: function, method, class constant) [deprecated-on-invalid-target]
class A {
    #[Deprecated]
//    ^^^^^^^^^^ CompileError: Attribute "Deprecated" cannot target property (allowed targets: function, method, class constant) [attribute-target-mismatch]
    public $p;
//  ^^^^^^^^^ MissingPropertyType: Property A::$p has no type annotation
}
#[AllowDynamicProperties]
//^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot apply #[AllowDynamicProperties] to trait T [allow-dynamic-properties-on-invalid-class]
trait T {}
class B {
    abstract public $x { get; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class B contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (B::$x::get) [abstract-member-in-concrete-class]
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property B::$x has no type annotation
}
