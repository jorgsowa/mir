===description===
Child redeclares a readonly parent property as readonly too — no error
===file===
<?php
class A {
    public readonly int $x;
    public function __construct(int $x) {
        $this->x = $x;
    }
}

class B extends A {
    public readonly int $x;
    public function __construct(int $x) {
        $this->x = $x;
    }
}
===expect===
