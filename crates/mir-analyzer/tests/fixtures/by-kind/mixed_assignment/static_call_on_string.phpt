===description===
Static call on string
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
MixedAssignment@9:1-9:17: Variable $b is assigned a mixed type
