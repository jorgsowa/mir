===description===
foreachInitialNull
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
// $result should be string|null, not mixed
echo $result;
===expect===
