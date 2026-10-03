===description===
null coalesce (??) should not emit UndefinedVariable for the LHS
$x ?? 'fallback' is valid PHP even if $x is undefined
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): string {
    return $x ?? 'fallback';
}
===expect===
