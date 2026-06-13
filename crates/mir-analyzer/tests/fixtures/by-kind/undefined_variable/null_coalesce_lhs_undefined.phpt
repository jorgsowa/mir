===description===
null coalesce (??) should not emit UndefinedVariable for the LHS
$x ?? 'fallback' is valid PHP even if $x is undefined
===config===
suppress=MixedReturnStatement
===file===
<?php
function test(): string {
    return $x ?? 'fallback';
}
===expect===
