===description===
Basic
===file===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
class Incomplete extends Base {}
===expect===
UnimplementedAbstractMethod@5:0: Class Incomplete must implement abstract method doWork()
