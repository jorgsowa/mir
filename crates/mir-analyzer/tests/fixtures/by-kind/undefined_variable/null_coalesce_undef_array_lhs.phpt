===description===
null coalesce (??) with undefined array variable on LHS — should not emit UndefinedVariable
$undefinedArr['key'] ?? d is valid PHP
===config===
suppress=MixedArrayAccess,MixedReturnStatement
===file===
<?php
function test(): string {
    return $undefinedArr['key'] ?? 'default';
}
===expect===
