===source===
<?php
class ParentClass {
    public function doStuff(): void {}
}
class Child extends ParentClass {
    private function doStuff(): void {}
}
===expect===
OverriddenMethodAccess: <no snippet>
