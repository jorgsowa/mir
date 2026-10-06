===description===
MagicMethodParamTypesCheckedForInterfaces
===file===
<?php
interface A
{
    public function a(string $className): int;
}

/**
 * @method int a(int $a)
// ^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::a() signature mismatch: parameter $a type 'int' is incompatible with parent type 'string'
 */
interface B extends A {}
