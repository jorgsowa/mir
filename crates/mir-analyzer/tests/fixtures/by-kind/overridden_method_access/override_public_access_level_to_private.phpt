===description===
Override public access level to private
===file===
<?php
class A {
    public function fooFoo(): void {}
}

class B extends A {
    private function fooFoo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Method B::foofoo() overrides with less visibility
}
