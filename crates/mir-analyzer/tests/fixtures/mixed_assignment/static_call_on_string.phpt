===description===
staticCallOnString
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
MixedAssignment
===ignore===
TODO
