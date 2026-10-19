===description===
Any call may rewrite a `global` import, so literal narrowing of it does not
survive a call.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$g = 'a';
function setGlobal(): void { global $g; $g = 'z'; }

function single(): void {
    global $g;
    if ($g === 'a') { setGlobal(); if ($g === 'z') {} }
}

function orChain(): void {
    global $g;
    if ($g === 'a' || $g === 'b') { setGlobal(); if ($g === 'z') {} }
}

function inArray(): void {
    global $g;
    if (in_array($g, ['a', 'b'], true)) { setGlobal(); if ($g === 'z') {} }
}

function noCall(): void {
    global $g;
    if ($g === 'a') { if ($g === 'z') {} }
}

function localNotGlobal(): void {
    $l = 'a';
    setGlobal();
    if ($l === 'z') {}
}
===expect===
ImpossibleIdenticalComparison@22:26-22:36: '===' between '"a"' and '"z"' is always false — these types can never be identical
ImpossibleIdenticalComparison@28:8-28:18: '===' between '"a"' and '"z"' is always false — these types can never be identical
