===description===
`isset($a['key'])`, `isset($a[$k])` and `isset($list[0])` on a generic array
with a nullable value type prove the element non-null inside the guard.
===file===
<?php
function takesInt(int $value): void { echo $value; }

/** @param array<string, int|null> $a */
function byKey(array $a, string $k): void {
    if (isset($a['key'])) {
        takesInt($a['key']);
    }
    if (isset($a[$k])) {
        takesInt($a[$k]);
    }
}

/** @param list<int|null> $list */
function byIndex(array $list): void {
    if (isset($list[0])) {
        takesInt($list[0]);
    }
}

/** @param array<string, int|null>|null $a */
function nullableContainer(?array $a): void {
    if (isset($a['key'])) {
        takesInt($a['key']);
    }
}
