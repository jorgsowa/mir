===source===
<?php
class Base {
    protected function method(): void {}
}
class Child extends Base {
    public function method(): void {}
}
===expect===
