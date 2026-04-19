===source===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
class Incomplete extends Base {}
===expect===
UnimplementedAbstractMethod: Class Incomplete must implement abstract method dowork()
