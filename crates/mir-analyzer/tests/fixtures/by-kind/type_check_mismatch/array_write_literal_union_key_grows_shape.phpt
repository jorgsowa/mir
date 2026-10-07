===description===
`$arr[$k] = $v` with `$k` a union of literal string keys grows the shape by each
key (as Psalm does): optional on an empty array outside a loop, definite otherwise.
Int keys keep the generic path.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array{first?: mixed, second?: mixed} $in
 * @return array{first?: int, second?: int}
 */
function optionsFromLoop(array $in): array {
    $options = [];
    foreach (['first', 'second'] as $name) {
        $value = $in[$name] ?? null;
        if (!\is_int($value) || $value < 1) {
            continue;
        }
        $options[$name] = $value;
    }
    return $options;
}

/** @param 'a'|'b' $k */
function emptyBase(string $k, int $v): void {
    $o = [];
    $o[$k] = $v;
    /** @mir-check $o is array{a?: int, b?: int} */
    $_ = $o;
}

/** @param 'a'|'b' $k */
function populatedBase(string $k, int $v): void {
    $o = ['x' => 1, 'a' => 2];
    $o[$k] = $v;
    /** @mir-check $o is array{x: 1, a: 2|int, b: int} */
    $_ = $o;
}

function loopOverShape(string $s): int {
    $points = ['left' => $s, 'right' => $s];
    $out = [];
    foreach ($points as $side => $p) {
        $out[$side] = ['len' => strlen($p)];
    }
    /** @mir-check $out is array{left: array{len: int<0, max>}, right: array{len: int<0, max>}} */
    return $out['left']['len'];
}

/** @param 1|2 $k */
function intKeys(int $k, int $v): void {
    $o = [];
    $o[$k] = $v;
    /** @mir-check $o is array<int, int> */
    $_ = $o;
}

/** @param 'a'|string $k */
function nonLiteralKey(string $k, int $v): void {
    $o = [];
    $o[$k] = $v;
    /** @mir-check $o is array<string, int> */
    $_ = $o;
}
