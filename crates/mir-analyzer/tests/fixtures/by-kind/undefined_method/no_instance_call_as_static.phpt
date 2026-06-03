===description===
No instance call as static
===file===
<?php
class C {
    public function foo() : void {}
}

(new C)::foo();
===expect===
InvalidStaticInvocation@6:1-6:15: Non-static method C::foo() cannot be called statically
