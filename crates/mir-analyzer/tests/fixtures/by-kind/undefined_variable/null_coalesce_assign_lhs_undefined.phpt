===description===
null coalesce assign (??=) should not emit UndefinedVariable for the target
$x ??= 'default' is valid PHP even if $x is undefined
===config===
suppress=MixedReturnStatement
===file===
<?php
function test(): string {
    $x ??= 'default';
    return $x;
}
===expect===
