===description===
self, static, and parent references are never reported as wrong case.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    public static function create(): static { return new static(); }
    public function getParent(): self { return $this; }
}
class Child extends Base {
    public function test(): void {
        $x = new self();
        $y = new static();
        $z = new parent();
    }
}
===expect===
