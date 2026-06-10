===description===
Method call on a parent-typed parameter resolves via parent class
===file===
<?php
class Base {
    public function getFoo(): string { return "x"; }
}
class Child extends Base {
    public function test(parent $x): void {
        $result = $x->getFoo();
        /** @mir-check $result is string */
        $_ = $result;
    }
}
===expect===
