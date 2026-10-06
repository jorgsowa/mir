===description===
Override public access level to protected
===file===
<?php
class A {
    public function fooFoo(): void {}
}

class B extends A {
    protected function fooFoo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Method B::foofoo() overrides with less visibility
}
