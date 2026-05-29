===description===
Foreach union method call
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
// After loop, $result is string|null
// This should error because null doesn't have strlen
$len = strlen($result);
===expect===
PossiblyNullArgument@15:15-15:22: Argument $string of strlen() might be null
