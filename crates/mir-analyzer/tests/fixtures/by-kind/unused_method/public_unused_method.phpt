===description===
Public unused method
===ignore===
TODO
===file===
<?php
final class A {
    /** @return void */
    public function foo() {}
}

new A();
===expect===
