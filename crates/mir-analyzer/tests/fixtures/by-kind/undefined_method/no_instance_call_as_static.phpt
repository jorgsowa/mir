===description===
No instance call as static
===file===
<?php
class C {
    public function foo() : void {}
}

(new C)::foo();
===expect===
InvalidStaticInvocation
===ignore===
TODO
