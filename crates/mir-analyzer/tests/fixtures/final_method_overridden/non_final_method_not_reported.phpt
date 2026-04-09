===source===
<?php
class Base {
    public function open(): void {}
}
class Child extends Base {
    public function open(): void {}
}
===expect===
