===description===
dynamic function call not reported
===file===
<?php
function test(): string {
    $fn = static fn(): string => 'hello';
    return $fn();
}
===expect===
