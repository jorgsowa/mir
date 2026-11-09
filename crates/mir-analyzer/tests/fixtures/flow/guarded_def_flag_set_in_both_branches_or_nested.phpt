===description===
A flag first assigned in both branches of an if/else, or set in a nested if, proves the
variable defined beside it wherever the flag holds; not when the else branch can set it too.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function uninitialisedFlag(object $o): void {
    if ($o instanceof Countable) { $n = 1; $has = true; } else { $has = false; }
    if ($has) { echo $n; }
}

function nestedFlag(bool $c, bool $d): void {
    $g = false;
    if ($c) { $n = 1; if ($d) { $g = true; } }
    if ($g) { echo $n; }
}

function elseSetsSameValue(object $o): void {
    if ($o instanceof Countable) { $n = 1; $has = true; } else { $has = rand(0, 1) === 1; }
    if ($has) { echo $n; }
//                   ^^ PossiblyUndefinedVariable: Variable $n might not be defined
}
