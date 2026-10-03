===description===
Variable callable (TClosure from closure literal): by-ref param writes the declared
type back to the caller's variable. No UndefinedVariable on a fresh $out.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = function(string &$out): void {
    $out = "hello";
};

$fn($result);
/** @mir-check $result is string */
$_ = $result;
===expect===
