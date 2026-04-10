===source===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
class Complete extends Base {
    public function doWork(): void {}
}
===expect===
