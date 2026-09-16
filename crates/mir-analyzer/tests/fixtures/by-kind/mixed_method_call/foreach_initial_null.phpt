===description===
Foreach initial null
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
/** @mir-check $result is string */
echo $result;
===expect===
UnusedVariable@9:0-9:7: Variable $result is never read
