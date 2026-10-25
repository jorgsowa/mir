===description===
Override protected access level to private
===file===
<?php
class A {
    protected function fooFoo(): void {}
}

class B extends A {
    private function fooFoo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Access level to B::fooFoo() must be protected (as in class A) or weaker
}
