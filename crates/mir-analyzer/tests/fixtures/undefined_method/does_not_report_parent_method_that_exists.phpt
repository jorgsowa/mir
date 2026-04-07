===source===
<?php
class Base {
    public function run(): void {}
}
class Child extends Base {
    public function doWork(): void {
        parent::run();
    }
}
===expect===
