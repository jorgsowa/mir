===description===
`@param T[] ...$x` describes each argument (`$x` is `list<T[]>`); only an int-keyed `array<int, T>` / `list<T>` is the aggregate.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string[] ...$groups */
function merge(...$groups): void {
    /** @mir-check $groups is list<array<int|string, string>> */
    $_ = $groups;
}

merge(['a'], ['b', 'c']);
merge();
merge('a');
//    ^^^ InvalidArgument: Argument $groups of merge() expects 'array<int|string, string>', got '"a"'
merge(['a'], [1]);
//           ^^^ InvalidArgument: Argument $groups of merge() expects 'array<int|string, string>', got 'array{0: 1}'

final class Bag {
    /** @param int[] ...$sets */
    public function add(...$sets): void {}
    /** @param int[] ...$sets */
    public static function make(...$sets): void {}
}
(new Bag())->add([1], [2]);
(new Bag())->add(1);
//               ^ InvalidArgument: Argument $sets of add() expects 'array<int|string, int>', got '1'
Bag::make([1]);
Bag::make(1);
//        ^ InvalidArgument: Argument $sets of make() expects 'array<int|string, int>', got '1'

/** @param int ...$bare */
function bare(...$bare): void {
    /** @mir-check $bare is list<int> */
    $_ = $bare;
}
bare(1, 2);
bare([1]);
//   ^^^ InvalidArgument: Argument $bare of bare() expects 'int', got 'array{0: 1}'

/** @param list<int> ...$lists */
function aggregate(...$lists): void {
    /** @mir-check $lists is list<int> */
    $_ = $lists;
}
aggregate(1, 2);
/** @param int[] ...$ids */
function nativeArray(array ...$ids): void {
    /** @mir-check $ids is list<array<int|string, int>> */
    $_ = $ids;
}
nativeArray([1], [2]);
