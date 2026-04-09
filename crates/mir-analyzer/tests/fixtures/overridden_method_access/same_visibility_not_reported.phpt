===source===
<?php
class Base {
    protected function method(): void {}
}
class Child extends Base {
    protected function method(): void {}
}
===expect===
