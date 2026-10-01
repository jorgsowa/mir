===description===
A `@return $this` method keeps the receiver's class, so the next call in the chain resolves on the subclass.
===cursor===
definition
===file===
<?php
class Base {
    /** @return $this */
    public function chain() { return $this; }
}
class Sub extends Base {
    public function subOnly(): void {}
}
function run(Sub $s): void {
    $s->chain()->subOn<CURSOR>ly();
}
===expect===
test.php@7:4-7:38
