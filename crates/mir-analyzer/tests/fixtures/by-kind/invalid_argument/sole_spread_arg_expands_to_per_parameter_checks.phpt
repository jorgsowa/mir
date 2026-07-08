===description===
A sole spread argument over a literal, sequentially-keyed shape must be
expanded into one binding per element so every parameter is checked
individually, not just the first — a single merged spread element type
previously bound only to the first parameter and silently skipped the rest.
===config===
suppress=UnusedParam
===file===
<?php
function needsTwoInts(int $a, int $b): void {}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_function(array $pair): void {
    needsTwoInts(...$pair);
}

class Calc {
    public static function needsTwoInts(int $a, int $b): void {}
}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_static_call(array $pair): void {
    Calc::needsTwoInts(...$pair);
}

class Pair {
    public function __construct(int $a, int $b) {}
}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_constructor(array $pair): void {
    new Pair(...$pair);
}
===expect===
InvalidArgument@8:18-8:25: Argument $b of needsTwoInts() expects 'int', got 'string'
InvalidArgument@19:24-19:31: Argument $b of needsTwoInts() expects 'int', got 'string'
InvalidArgument@30:14-30:21: Argument $b of Pair::__construct() expects 'int', got 'string'
