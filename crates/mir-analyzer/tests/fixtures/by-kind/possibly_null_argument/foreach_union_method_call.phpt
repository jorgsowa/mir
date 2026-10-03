===description===
Foreach union method call
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Item {
    public function transform(): string {
        return "result";
    }
}

$items = [new Item()];
$result = null;
foreach ($items as $item) {
    $result = $item->transform();
}
// The literal array is non-empty, so the loop assigns string on every path.
$len = strlen($result);
===expect===
