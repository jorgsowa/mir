===description===
The warm pass resolves the `App\strtoupper` fallback in A first, so B reuses
that name. After an edit to its caller, B's inferred return type is
re-verified before anything resolves the name again.
===file:A.php===
<?php
namespace App;
class A { public function m(string $s) { return strtoupper($s); } }
===file:B.php===
<?php
namespace App;
class B { public function m(string $s) { return strtoupper($s); } }
===file:C.php===
<?php
namespace App;
function c(): int { return (new B())->m('x'); }
===expect===
C.php: InvalidReturnType@3:20-3:45: Return type 'string' is not compatible with declared 'int'
===edit:C.php===
<?php
namespace App;
function c(): int { return (new B())->m('y'); }
===expect===
C.php: InvalidReturnType@3:20-3:45: Return type 'string' is not compatible with declared 'int'
