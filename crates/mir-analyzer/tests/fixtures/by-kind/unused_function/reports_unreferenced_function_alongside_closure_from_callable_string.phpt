===description===
a function not passed to Closure::fromCallable is still reported unused even when another function is
===config===
suppress=
===file===
<?php
function helper(): void {}
function unused(): void {}

Closure::fromCallable('helper');
===expect===
UnusedFunction@3:0-3:26: Function unused() is never called
