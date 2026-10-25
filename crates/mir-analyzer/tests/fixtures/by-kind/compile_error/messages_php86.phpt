===description===
PHP 8.6's texts.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
class A {
    public private(set) int $x { get => 1; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: get-only virtual property A::$x must not specify asymmetric visibility [virtual-property-asymmetric-visibility]
}
new A(?);
//<^^^^^^^^ TooManyArguments: Too many arguments for A::__construct(): expected 0, got 1
//    ^ CompileError: Cannot create Closure for new expression [partial-application-in-new]
