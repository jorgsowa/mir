===description===
An open shape's KNOWN properties still must satisfy array<K,V>'s value type
even though the shape may carry extra unknown keys — the is_open flag only
excuses the unknown keys, not the ones already declared.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $arr */
function wantsIntMap(array $arr): void {}

/** @param array{a: string, ...} $shape */
function passBadOpenShape(array $shape): void {
    wantsIntMap($shape);
}

/** @param array{a: int, ...} $shape */
function passGoodOpenShape(array $shape): void {
    wantsIntMap($shape);
}

/** @param non-empty-array<string, int> $arr */
function wantsNonEmptyIntMap(array $arr): void {}

/** @param array{a: string, ...} $shape */
function passBadOpenShapeNonEmpty(array $shape): void {
    wantsNonEmptyIntMap($shape);
}
===expect===
InvalidArgument@7:16-7:22: Argument $arr of wantsIntMap() expects 'array<string, int>', got 'array{'a': string}'
InvalidArgument@20:24-20:30: Argument $arr of wantsNonEmptyIntMap() expects 'non-empty-array<string, int>', got 'array{'a': string}'
