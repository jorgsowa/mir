===description===
A flag set to `false` beside a definition proves it wherever the flag is falsy again,
also when the branch that leaves the flag falsy defines the variable itself; not when
the flag may already be false.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function falseFlag(bool $a): void {
    $ok = true;
    if ($a) { $ok = false; $msg = 'a'; }
    if (!$ok) { echo $msg; }
    if ($ok === false) { echo $msg; }
}

function otherBranchDefinesIt(bool $a, bool $b): void {
    $ok = extension_loaded('pdo');
    if (!$ok) { $msg = 'none'; }
    else {
        if ($a) { $ok = false; $msg = 'a'; }
        if ($b) { $ok = false; $msg = 'b'; }
    }
    if (!$ok) { echo $msg; }
}

function flagMayAlreadyBeFalse(bool $a): void {
    $ok = rand(0, 1) === 1;
    if ($a) { $ok = false; $msg = 'a'; }
    if (!$ok) { echo $msg; }
//                   ^^^^ PossiblyUndefinedVariable: Variable $msg might not be defined
}
