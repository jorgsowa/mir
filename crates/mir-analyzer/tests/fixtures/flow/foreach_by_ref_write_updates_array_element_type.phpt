===description===
Assigning to a by-reference foreach value changes the iterated array's element type
===file===
<?php
/**
 * @param list<int|string> $list
 * @param array<string, int> $map
 * @return list<string>
 */
function f(array $list, array $map): array {
    foreach ($list as &$item) {
        $item = (string) $item;
    }
    unset($item);
    foreach ($map as &$value) {
        $value = $value > 0 ? 'yes' : null;
    }
    unset($value);
    /** @mir-check $map is array<string, "yes"|null> */
    return $list;
}
