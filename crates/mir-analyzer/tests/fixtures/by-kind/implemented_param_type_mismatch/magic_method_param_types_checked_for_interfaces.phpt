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
 */
interface B extends A {}
//<^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::a() signature mismatch: parameter $a type 'int' is incompatible with parent type 'string'

===expect===
