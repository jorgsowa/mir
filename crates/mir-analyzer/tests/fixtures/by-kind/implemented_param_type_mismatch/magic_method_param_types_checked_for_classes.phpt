===description===
MagicMethodParamTypesCheckedForClasses
===config===
suppress=UnusedParam
===file===
<?php
class A
{
    public function a(int $className): int { return 0; }
}

/**
 * @method int a(string $a)
 */
class B extends A {}

===expect===
MethodSignatureMismatch@10:0-10:20: Method B::a() signature mismatch: parameter $a type 'string' is narrower than parent type 'int'
