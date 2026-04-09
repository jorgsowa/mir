===source===
<?php
final class Base {
    public function hello(): void {}
}
class Child extends Base {}
===expect===
FinalClassExtended: <no snippet>
