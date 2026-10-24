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
//                        ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}

function localNotGlobal(): void {
    $l = 'a';
    setGlobal();
    if ($l === 'z') {}
//      ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}
