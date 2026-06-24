===description===
Calling a variable callable with a fresh (undefined) variable passed to a by-ref
param must not emit UndefinedVariable — the pre-marking runs before arg analysis.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
$writer = function(int &$x): void { $x = 42; };

// $fresh has never been assigned — should be pre-marked by the by-ref param.
$writer($fresh);
/** @mir-check $fresh is int */
$_ = $fresh;
===expect===
