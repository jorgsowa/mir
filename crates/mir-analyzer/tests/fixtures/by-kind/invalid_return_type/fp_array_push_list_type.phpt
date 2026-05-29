===description===
Array push [] on existing empty array produces list<T>, not array<mixed, T>
===file===
<?php

/**
 * @param string[] $items
 * @return list<string>
 */
function buildList(array $items): array
{
    $out = [];
    foreach ($items as $item) {
        $out[] = $item;
    }
    /** @mir-check $out is list<string> */
    return $out;
}
===expect===
