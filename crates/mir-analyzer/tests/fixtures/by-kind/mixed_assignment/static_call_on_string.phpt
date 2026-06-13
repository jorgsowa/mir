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
UnusedPsalmSuppress@9:0-9:0: Suppress annotation for 'InvalidStringClass' is never used
MixedAssignment@9:1-9:17: Variable $b is assigned a mixed type
