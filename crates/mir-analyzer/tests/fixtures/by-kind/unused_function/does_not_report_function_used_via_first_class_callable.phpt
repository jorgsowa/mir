===description===
A free function used only through first-class-callable syntax (`helper(...)`)
must not be reported unused.
===config===
suppress=UnusedVariable
===file===
<?php
function helper(): void {}

$c = helper(...);
$c();
===expect===
