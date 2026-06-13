===description===
Foreach unknown array size
===config===
suppress=MixedAssignment
===file===
<?php
function getItems(): array {
    return [];
}

$items = getItems(); // Unknown size and type at runtime
$result = null;
foreach ($items as $item) {
    // $item type is mixed since array is unknown
    $result = $item->transform();
}
// After loop, $result is mixed|null
// because array size is unknown and loop might not execute
echo $result;
===expect===
MixedMethodCall@10:15-10:33: Method transform() called on mixed type
