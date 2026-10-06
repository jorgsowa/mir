===description===
Basic
===file===
<?php
class ParentClass {
    public function doStuff(): void {}
}
class Child extends ParentClass {
    private function doStuff(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Method Child::dostuff() overrides with less visibility
}
