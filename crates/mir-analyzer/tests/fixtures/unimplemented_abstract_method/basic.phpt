===source===
<?php
abstract class Base {
    abstract public function required(): void;
}
class Incomplete extends Base {
}
===expect===
UnimplementedAbstractMethod: class Incomplete extends Base {
}
