===description===
a function used only as a bare string argument to a user function with a plain `callable` param must not be reported unused
===config===
suppress=
===file===
<?php
function helper(): void {}

function invokeCallback(callable $cb): void {
    $cb();
}

invokeCallback('helper');
===expect===
