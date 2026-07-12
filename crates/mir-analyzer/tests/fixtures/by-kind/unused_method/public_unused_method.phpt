===description===
Public unused method
===config===
suppress=
===file===
<?php
final class A {
    /** @return void */
    public function foo() {}
}

new A();
===expect===
