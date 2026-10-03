===description===
null coalesce (??) with undefined array variable on LHS — should not emit UndefinedVariable
$undefinedArr['key'] ?? d is valid PHP
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): string {
    return $undefinedArr['key'] ?? 'default';
}
===expect===
