===description===
No instance call as static
===file===
<?php
class C {
    public function foo() : void {}
}

(new C)::foo();
//<^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method C::foo() cannot be called statically
===expect===
