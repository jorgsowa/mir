===description===
An attributed closure is reported from its function keyword, not from the attribute.
===file===
<?php
$f = #[\Deprecated]
    function (int $x) { return $x; };
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingClosureReturnType: Closure has no return type annotation
$f(1);
