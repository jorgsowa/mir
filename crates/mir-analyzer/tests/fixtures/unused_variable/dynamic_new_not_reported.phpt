===description===
dynamic new not reported
===file===
<?php
function test(): object {
    $class = 'stdClass';
    return new $class();
}
===expect===
