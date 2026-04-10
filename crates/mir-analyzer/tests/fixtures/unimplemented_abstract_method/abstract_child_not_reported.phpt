===source===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
abstract class StillAbstract extends Base {}
===expect===
