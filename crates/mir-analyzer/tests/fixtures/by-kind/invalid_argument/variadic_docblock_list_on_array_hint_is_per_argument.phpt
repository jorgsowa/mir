===description===
`@param list<S> ...$x` on a native `array ...$x` describes each argument; the aggregate reading stays for non-array native hints and array-valued element types.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<int> ...$batches */
function run(array ...$batches): void {
    /** @mir-check $batches is list<list<int>> */
    $_ = $batches;
}

run([1, 2], [3]);
run();
run([1], ['a']);
//       ^^^^^ InvalidArgument: Argument $batches of run() expects 'list<int>', got 'array{0: "a"}'
run(1);
//  ^ InvalidArgument: Argument $batches of run() expects 'list<int>', got '1'

/** @param list<int> $lists */
function spread(array ...$lists): void {}
/** @var list<list<int>> $groups */
$groups = [[1], [2]];
spread(...$groups);

final class Runner {
    /** @param array<int, string> ...$rows */
    public function __construct(array ...$rows) {}

    /** @param non-empty-list<int> ...$batches */
    public function add(array ...$batches): void {}

    /** @param list<string> ...$names */
    public static function make(array ...$names): void {}
}
new Runner(['a'], ['b']);
new Runner([1]);
//         ^^^ InvalidArgument: Argument $rows of Runner::__construct() expects 'array<int, string>', got 'array{0: 1}'
(new Runner([]))->add([1], [2, 3]);
(new Runner([]))->add(1);
//                    ^ InvalidArgument: Argument $batches of add() expects 'non-empty-list<int>', got '1'
Runner::make(['x']);
Runner::make('x');
//           ^^^ InvalidArgument: Argument $names of make() expects 'list<string>', got '"x"'

// Unchanged: an array-valued element type is ambiguous and stays the aggregate.
/** @param list<array<string, int>> $maps */
function maps(array ...$maps): void {
    /** @mir-check $maps is list<array<string, int>> */
    $_ = $maps;
}
maps(['a' => 1], ['b' => 2]);

// Unchanged: a string-keyed array is already per-argument.
/** @param array<string, int> ...$maps */
function stringKeyed(array ...$maps): void {
    /** @mir-check $maps is list<array<string, int>> */
    $_ = $maps;
}
stringKeyed(['a' => 1]);
