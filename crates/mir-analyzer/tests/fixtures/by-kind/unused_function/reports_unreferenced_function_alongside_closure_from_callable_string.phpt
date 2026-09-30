===description===
a function not passed to Closure::fromCallable is still reported unused even when another function is
===config===
suppress=
===file===
<?php
function helper(): void {}
function unused(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function unused() is never called

Closure::fromCallable('helper');
===expect===
