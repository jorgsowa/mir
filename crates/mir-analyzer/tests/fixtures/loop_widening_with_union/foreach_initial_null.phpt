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
/** @mir-check $result is string|null */
echo $result;
===expect===
