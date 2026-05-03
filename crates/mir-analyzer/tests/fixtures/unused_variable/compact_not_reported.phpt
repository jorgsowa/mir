===description===
compact not reported
===file===
<?php
function foo(): array {
    $name = 'Alice';
    $age = 30;
    return compact('name', 'age');
}
===expect===
===ignore===
TODO
