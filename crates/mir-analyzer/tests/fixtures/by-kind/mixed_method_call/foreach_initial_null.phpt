===description===
An initialization overwritten by a foreach that is known to execute is not an
unused variable when the foreach result is read afterwards.
===config===
suppress=
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
