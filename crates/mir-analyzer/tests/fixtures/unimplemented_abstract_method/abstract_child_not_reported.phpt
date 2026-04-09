===source===
<?php
abstract class Base {
    abstract public function required(): void;
}
abstract class StillAbstract extends Base {
}
===expect===
