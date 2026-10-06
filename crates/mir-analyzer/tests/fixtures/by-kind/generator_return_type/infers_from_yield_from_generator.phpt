===description===
`yield from $anotherGenerator` contributes the delegated generator's own
key/value type params, including when the delegate is an unannotated
function in the same file whose type is inferred on demand.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function inner() {
    yield 'k' => 1;
}

function outer() {
    yield from inner();
}

$g = outer();
/** @mir-check $g is Generator<"k", 1, mixed, void> */
$_ = 1;
