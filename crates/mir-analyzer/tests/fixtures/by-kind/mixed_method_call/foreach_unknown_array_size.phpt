===description===
Foreach unknown array size
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//            ^^^^^^^^^^^^^^^^^^ MixedMethodCall: Method transform() called on mixed type
}
// After loop, $result is mixed|null
// because array size is unknown and loop might not execute
echo $result;
