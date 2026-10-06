===description===
A sole spread argument over a literal, sequentially-keyed shape must be
expanded into one binding per element so every parameter is checked
individually, not just the first — a single merged spread element type
previously bound only to the first parameter and silently skipped the rest.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function needsTwoInts(int $a, int $b): void {}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_function(array $pair): void {
    needsTwoInts(...$pair);
//                ^^^^^^^ InvalidArgument: Argument $b of needsTwoInts() expects 'int', got 'string'
}

class Calc {
    public static function needsTwoInts(int $a, int $b): void {}
}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_static_call(array $pair): void {
    Calc::needsTwoInts(...$pair);
//                      ^^^^^^^ InvalidArgument: Argument $b of needsTwoInts() expects 'int', got 'string'
}

class Pair {
    public function __construct(int $a, int $b) {}
}

/**
 * @param array{0: int, 1: string} $pair
 */
function via_constructor(array $pair): void {
    new Pair(...$pair);
//            ^^^^^^^ InvalidArgument: Argument $b of Pair::__construct() expects 'int', got 'string'
}
