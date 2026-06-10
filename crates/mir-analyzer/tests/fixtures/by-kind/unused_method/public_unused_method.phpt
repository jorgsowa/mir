===description===
Public unused method
===file===
<?php
final class A {
    /** @return void */
    public function foo() {}
}

new A();
===expect===
