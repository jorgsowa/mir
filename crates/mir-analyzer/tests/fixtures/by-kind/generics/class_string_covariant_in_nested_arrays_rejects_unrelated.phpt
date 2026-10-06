===description===
class-string<T> nested in arrays is still rejected when T is unrelated or a supertype
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class P {}
class C extends P {}
class Other {}
interface I {}

/** @param array{k: class-string<C>} $c */
function shape(array $c): void {}
/** @param list<class-string<C>> $c */
function lst(array $c): void {}
/** @param array<string, class-string<I>> $c */
function map(array $c): void {}

/**
 * @param array{k: class-string<P>} $parent
 * @param array{k: class-string<Other>} $other
 * @param list<class-string<P>> $parentList
 * @param array<string, class-string<P>> $parentMap
 */
function bad($parent, $other, $parentList, $parentMap): void {
    shape($parent);
//        ^^^^^^^ InvalidArgument: Argument $c of shape() expects 'array{'k': class-string<C>}', got 'array{'k': class-string<P>}'
    shape($other);
//        ^^^^^^ InvalidArgument: Argument $c of shape() expects 'array{'k': class-string<C>}', got 'array{'k': class-string<Other>}'
    lst($parentList);
//      ^^^^^^^^^^^ InvalidArgument: Argument $c of lst() expects 'list<class-string<C>>', got 'list<class-string<P>>'
    map($parentMap);
//      ^^^^^^^^^^ InvalidArgument: Argument $c of map() expects 'array<string, class-string<I>>', got 'array<string, class-string<P>>'
}
