===description===
MagicMethodReturnTypesCheckedForClasses
===config===
suppress=UnusedParam
===file===
<?php
class A
{
    public function a(int $className): int { return 0; }
}

/**
 * @method stdClass a(int $a)
 */
class B extends A {}

===expect===
