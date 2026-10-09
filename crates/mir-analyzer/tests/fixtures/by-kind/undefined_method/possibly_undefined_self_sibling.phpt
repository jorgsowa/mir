===description===
`static`/`self` atoms in the union count as members declaring the method.
===file===
<?php
class Base {
    public function ping(): void {}
}
class Other {}
class Holder extends Base {
    /** @return static|Other */
    public function pick() { return $this; }
}
function test(Holder $h): void {
    $h->pick()->ping();
//  ^^^^^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method Other::ping() might not exist
}
