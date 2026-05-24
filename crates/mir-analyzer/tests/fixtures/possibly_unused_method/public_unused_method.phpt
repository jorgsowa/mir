===description===
publicUnusedMethod
===file===
<?php
final class A {
    /** @return void */
    public function foo() {}
}

new A();
===expect===
PossiblyUnusedMethod
===ignore===
TODO
