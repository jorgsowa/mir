===source===
<?php
class ParentClass {
    final public function locked(): void {}
}
class Child extends ParentClass {
    public function locked(): void {}
}
===expect===
FinalMethodOverridden: public function locked(): void {}
