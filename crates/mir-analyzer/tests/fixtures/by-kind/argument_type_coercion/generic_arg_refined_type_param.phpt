===description===
A generic argument whose type arguments are broader than the param's refined ones (int vs
positive-int, nullability kept) is an ArgumentTypeCoercion. Unrelated or nullability-widening
type arguments stay InvalidArgument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
final class Box {
    /** @param T $v */
    public function __construct(public mixed $v) {}
}
/** @param Box<positive-int|null> $b */
function takes_refined_nullable(Box $b): void {}
/** @param Box<non-empty-string> $b */
function takes_refined_string(Box $b): void {}

/**
 * @param Box<int|null> $a
 * @param Box<int> $b
 * @param Box<string> $c
 * @param Box<null|int> $d
 * @param Box<non-empty-string> $e
 * @param Box<float> $f
 */
function run(Box $a, Box $b, Box $c, Box $d, Box $e, Box $f): void {
    takes_refined_nullable($a);
//                         ^^ ArgumentTypeCoercion: Argument $b of takes_refined_nullable() expects 'Box<positive-int|null>', got 'Box<int|null>' — coercion may fail at runtime
    takes_refined_nullable($d);
//                         ^^ ArgumentTypeCoercion: Argument $b of takes_refined_nullable() expects 'Box<positive-int|null>', got 'Box<null|int>' — coercion may fail at runtime
    takes_refined_nullable($b);
//                         ^^ ArgumentTypeCoercion: Argument $b of takes_refined_nullable() expects 'Box<positive-int|null>', got 'Box<int>' — coercion may fail at runtime
    takes_refined_string($c);
//                       ^^ ArgumentTypeCoercion: Argument $b of takes_refined_string() expects 'Box<non-empty-string>', got 'Box<string>' — coercion may fail at runtime
    takes_refined_string($e);
    takes_refined_string($a);
//                       ^^ InvalidArgument: Argument $b of takes_refined_string() expects 'Box<non-empty-string>', got 'Box<int|null>'
    takes_refined_nullable($f);
//                         ^^ InvalidArgument: Argument $b of takes_refined_nullable() expects 'Box<positive-int|null>', got 'Box<float>'
}
