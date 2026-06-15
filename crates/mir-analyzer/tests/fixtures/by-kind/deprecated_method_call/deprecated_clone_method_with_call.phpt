===description===
Deprecated clone method with call
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    /**
     * @deprecated
     */
    public function __clone() {
    }
}

$a = new Foo;
$aa = clone $a;
===expect===
DeprecatedMethodCall@11:6-11:14: Call to deprecated method Foo::__clone
