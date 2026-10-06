===description===
P3: First-class callable from an inherited method resolves through the ancestor chain.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

class Base {
    public function compute(int $x): float { return (float) $x; }
}

class Child extends Base {}

$c = new Child();
$fn = $c->compute(...);
/** @mir-check $fn is Closure(int): float */
$_ = $fn;
