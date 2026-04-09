===source===
<?php
function foo(): string {
    $name = 'world';
    return <<<EOT
Hello $name!
EOT;
}
===expect===
