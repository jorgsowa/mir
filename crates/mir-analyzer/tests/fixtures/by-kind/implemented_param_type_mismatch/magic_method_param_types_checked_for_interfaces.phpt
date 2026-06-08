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

===expect===
MethodSignatureMismatch@10:0-10:24: Method B::a() signature mismatch: parameter $a type 'int' is narrower than parent type 'string'
