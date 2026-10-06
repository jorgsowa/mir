===description===
Basic
===file===
<?php
abstract class Base {
    abstract public function doWork(): void;
}
class Incomplete extends Base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedAbstractMethod: Class Incomplete must implement abstract method doWork()
