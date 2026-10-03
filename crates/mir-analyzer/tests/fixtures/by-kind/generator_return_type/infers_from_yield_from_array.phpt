===description===
`yield from $array` contributes the array's own key/value types rather than
`mixed`/`mixed`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function gen() {
    yield from [1, 2, 3];
}

$g = gen();
/** @mir-check $g is Generator<int, 1|2|3, mixed, void> */
$_ = 1;
===expect===
