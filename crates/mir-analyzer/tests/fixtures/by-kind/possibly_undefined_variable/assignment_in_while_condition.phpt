===description===
An assignment in the `&&`-guarded `while` condition defines `$line` in the body.
===config===
suppress=UnusedForeachValue
php_version=8.4
===file===
<?php
function run(mixed $resource): void {
    while (!feof($resource) && ($line = fgets($resource))) {
        echo strlen($line);
    }
}
===expect===
