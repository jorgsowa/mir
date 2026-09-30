===description===
Go-to-definition on a call chained off a `: self` return resolves against the declaring class, not the calling subclass.
===cursor===
definition
===file===
<?php
class Base {
    public function returnsSelf(): self { return $this; }
    public function baseOnly(): void {}
}
class Sub extends Base {
    public function baseOnly(): void {}
}
(new Sub())->returnsSelf()->base<CURSOR>Only();
===expect===
test.php@4:4-4:39
