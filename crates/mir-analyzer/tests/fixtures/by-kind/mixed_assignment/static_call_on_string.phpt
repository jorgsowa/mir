===description===
Static call on string
===config===
suppress=UnusedVariable
===file===
<?php
class A {
    public static function bar(): int {
        return 5;
    }
}
$foo = "A";
/** @suppress InvalidStringClass */
$b = $foo::bar();
===expect===
MixedAssignment@9:0-9:16: Variable $b is assigned a mixed type
UnusedPsalmSuppress@9:0-9:0: Suppress annotation for 'InvalidStringClass' is never used
