===description===
closure parameter not undefined no error
===file===
<?php
$fn = function(string $name): string {
    return $name;
};
===expect===
===ignore===
TODO
