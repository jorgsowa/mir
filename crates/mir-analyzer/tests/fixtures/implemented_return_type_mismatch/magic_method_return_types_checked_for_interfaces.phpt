===description===
MagicMethodReturnTypesCheckedForInterfaces
===file===
<?php
interface A
{
    public function a(int $className): int;
}

/**
 * @method stdClass a(int $a)
 */
interface B extends A {}

===expect===
ImplementedReturnTypeMismatch
===ignore===
TODO
