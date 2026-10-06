===description===
An explicit `@return Generator<...>` docblock always wins over the inferred
type from the body's yields — inference only fills in when nothing is
declared.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return Generator<string, int> */
function gen() {
    yield 1 => 2;
}

$g = gen();
/** @mir-check $g is Generator<string, int> */
$_ = 1;
