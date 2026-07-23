===description===
An open, empty-properties shape (array{...}) may hold unknown, possibly
non-list, extra keys at runtime — it must not be silently dropped when
merged into a union with a list<T> across a branch, since (unlike list<T>
itself) it isn't provably list-compatible.
===config===
suppress=MissingPropertyType,MissingConstructor,UnusedParam
===file===
<?php

class Sink {
    /** @var list<int> */
    public array $x;
}

/**
 * @param array{...} $openArr
 * @param list<int> $listArr
 */
function test_open_shape_survives_merge(bool $cond, array $openArr, array $listArr, Sink $s): void {
    if ($cond) {
        $arr = $openArr;
    } else {
        $arr = $listArr;
    }
    $s->x = $arr;
}

/**
 * @param array{} $emptyArr
 * @param list<int> $listArr
 */
function test_closed_empty_shape_still_subsumed(bool $cond, array $emptyArr, array $listArr, Sink $s): void {
    if ($cond) {
        $arr = $emptyArr;
    } else {
        $arr = $listArr;
    }
    $s->x = $arr;
}
===expect===
InvalidPropertyAssignment@18:4-18:16: Property $x expects 'list<int>', cannot assign 'array{}|list<int>'
