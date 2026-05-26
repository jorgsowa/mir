===description===
null coalesce (??) with undefined array variable on LHS — should not emit UndefinedVariable
$undefinedArr['key'] ?? d is valid PHP
===file===
<?php
function test(): string {
    return $undefinedArr['key'] ?? 'default';
}
===expect===
