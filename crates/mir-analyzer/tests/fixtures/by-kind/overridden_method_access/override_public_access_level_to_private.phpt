===description===
Override public access level to private
===file===
<?php
class A {
    public function fooFoo(): void {}
}

class B extends A {
    private function fooFoo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Access level to B::fooFoo() must be public (as in class A)
}
