===source===
<?php
final class Base {
    public function hello(): void {}
}
class Child extends Base {}
===expect===
FinalClassExtended: class Child extends Base {}
