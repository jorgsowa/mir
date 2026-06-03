===description===
Deprecated clone method with call
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
DeprecatedMethodCall@11:7-11:15: Call to deprecated method Foo::__clone
