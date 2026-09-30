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
//<^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::a() signature mismatch: parameter $a type 'string' is incompatible with parent type 'int'

===expect===
