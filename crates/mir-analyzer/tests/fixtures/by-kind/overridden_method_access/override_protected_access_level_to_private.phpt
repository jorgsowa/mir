===description===
Override protected access level to private
===file===
<?php
class A {
    protected function fooFoo(): void {}
}

class B extends A {
    private function fooFoo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Method B::foofoo() overrides with less visibility
}
===expect===
