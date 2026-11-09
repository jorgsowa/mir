===description===
A guard proves only a variable its branch defines on every path, and it does so also
when the variable may already be defined, as on a later loop iteration.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function definedOnlySometimesInBranch(bool $g): void {
    if ($g) { if (rand(0, 1) === 1) { $x = 1; } }
    if ($g) { echo $x; }
//                 ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

/** @param list<int> $items */
function insideLoop(array $items): void {
    foreach ($items as $i) {
        if ($i > 0) {
            $c = $i === 1;
            if ($c) { $title = 'a'; }
            if (!$c) { $title = 'b'; }
        } else {
            $title = 'c';
        }
        echo $title;
    }
}
