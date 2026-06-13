===description===
closure parameter not undefined no error
===config===
suppress=UnusedVariable
===file===
<?php
$fn = function(string $name): string {
    return $name;
};
===expect===
