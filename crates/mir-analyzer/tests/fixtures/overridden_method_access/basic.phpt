===source===
<?php
class Base {
    public function visible(): void {}
}
class Child extends Base {
    private function visible(): void {}
}
===expect===
OverriddenMethodAccess: private function visible(): void {}
