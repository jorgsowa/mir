===description===
No instance call as static
===file===
<?php
class C {
    public function foo() : void {}
}

(new C)::foo();
===expect===
InvalidStaticInvocation@6:0-6:14: Non-static method C::foo() cannot be called statically
