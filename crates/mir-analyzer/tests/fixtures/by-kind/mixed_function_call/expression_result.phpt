===description===
MixedFunctionCall fires when the callee is the result of an expression that
evaluates to mixed.
===file===
<?php
/** @return mixed */
function getMixed(): mixed { return null; }

getMixed()();

===expect===
MixedFunctionCall@5:0-5:12: Cannot call mixed type as a function
