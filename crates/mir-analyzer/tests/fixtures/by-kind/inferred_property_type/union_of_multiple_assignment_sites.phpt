===description===
When both branches of an if/else assign different types to the same
untyped property, the inferred type is their union — `prop_refined` already
merges same as it does for any other narrowed property.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
class A {}
class B {}

class Holder {
    public $thing;

    public function __construct(bool $cond) {
        if ($cond) {
            $this->thing = new A();
        } else {
            $this->thing = new B();
        }
    }

    public function read(): void {
        /** @mir-check $this->thing is A|B */
        $_ = 1;
    }
}
===expect===
