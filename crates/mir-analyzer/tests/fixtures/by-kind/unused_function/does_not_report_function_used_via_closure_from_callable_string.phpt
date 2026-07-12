===description===
a function used only as a bare string argument to Closure::fromCallable must not be reported unused
===config===
suppress=
===file===
<?php
function helper(): void {}

Closure::fromCallable('helper');
===expect===
