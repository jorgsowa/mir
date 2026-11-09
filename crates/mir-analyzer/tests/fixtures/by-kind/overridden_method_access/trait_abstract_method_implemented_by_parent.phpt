===description===
A trait's abstract method is implemented by the inherited parent method, so calling it resolves to the parent and its visibility isn't narrowed.
===file===
<?php
abstract class P { public function n(): string { return ''; } }
trait T { abstract protected function n(): string; }
class C extends P { use T; }
function f(C $c): string { return $c->n(); }
