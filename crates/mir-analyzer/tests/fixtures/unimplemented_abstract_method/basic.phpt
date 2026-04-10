===source===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
class Incomplete extends Base {}
===expect===
UnimplementedAbstractMethod: <no snippet>
