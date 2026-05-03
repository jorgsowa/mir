===description===
empty check not reported
===file===
<?php
function foo(): bool {
    $items = [];
    return empty($items);
}
===expect===
===ignore===
TODO
