===description===
heredoc interpolation not reported
===file===
<?php
function foo(): string {
    $name = 'world';
    return <<<EOT
Hello $name!
EOT;
}
===expect===
===ignore===
TODO
