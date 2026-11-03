===description===
`static::`, `$this::` and `constant('Class::NAME')` read the constant
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    private const VIA_STATIC = 1;
    private const VIA_INSTANCE = 2;
    private const VIA_FUNCTION = 3;

    public function run(): int {
        return static::VIA_STATIC + $this::VIA_INSTANCE + constant('A::VIA_FUNCTION');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Error is thrown but not declared in @throws
    }
}

(new A())->run();
