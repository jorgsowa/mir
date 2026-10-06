===description===
Impure constructor call
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Bar;

class A {
    public int $a = 5;
}

class B {
    public function __construct(A $a) {
        $a->a++;
    }
}

/** @pure */
function filterOdd(int $i, A $a) : ?int {
    $b = new B($a);
//       ^^^^^^^^^ ImpureFunctionCall: Calling impure function Bar\B::__construct() in a @pure function

    if ($i % 2 === 0 || $a->a === 2) {
        return $i;
    }

    return null;
}
