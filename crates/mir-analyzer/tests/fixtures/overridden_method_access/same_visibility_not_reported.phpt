===source===
<?php
class ParentClass {
    protected function doStuff(): void {}
}
class Child extends ParentClass {
    protected function doStuff(): void {}
}
===expect===
