===source===
<?php
function foo(): bool {
    $items = [];
    return empty($items);
}
===expect===
