===source===
<?php
function test(): object {
    $class = 'stdClass';
    return new $class();
}
===expect===
