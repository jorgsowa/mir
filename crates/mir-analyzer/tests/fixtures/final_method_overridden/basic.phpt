===source===
<?php
class Base {
    final public function locked(): void {}
}
class Child extends Base {
    public function locked(): void {}
}
===expect===
FinalMethodOverridden: <no snippet>
