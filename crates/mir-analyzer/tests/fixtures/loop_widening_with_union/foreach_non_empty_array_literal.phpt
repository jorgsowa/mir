===description===
Foreach non empty array literal
===file===
<?php
class Item {
    public function transform(): string {
        return "result";
    }
}

// Array with exactly one element — loop guaranteed to execute
$result = null;
foreach ([new Item()] as $item) {
    $result = $item->transform();
}
// After loop, $result should be just string, not string|null
// because the loop is guaranteed to execute
if (is_string($result)) {
    echo $result;
}
===expect===
