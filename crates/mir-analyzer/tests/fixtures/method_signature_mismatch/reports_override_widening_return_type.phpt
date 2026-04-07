===source===
<?php
class Base {
    public function f(): int { return 1; }
}
class Child extends Base {
    public function f(): int|string { return 1; }
}
===expect===
MethodSignatureMismatch: f
