===source===
<?php
abstract class Base {
    abstract public function required(): void;
}
class Complete extends Base {
    public function required(): void {}
}
===expect===
