===description===
`@template T of Collection<int>&Countable` (and a plain
`Collection<int>&Countable $param`) rejects a `Collection<string>&Countable`
argument — the `(TNamedObject, TIntersection)` subtype arm previously
dropped each intersection part's own type args, checking only the bare
class name.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Countable {}

/** @template T */
interface Collection {}

/** @implements Collection<string> */
class StringCollection implements Collection, Countable {
    /** @return string */
    public function value() { return ''; }
}

/** @implements Collection<int> */
class IntCollection implements Collection, Countable {
    /** @return int */
    public function value() { return 0; }
}

/**
 * @template T of Collection<int>&Countable
 * @param T $t
 */
function needsIntCollectionBound($t): void {}

/** @param Collection<int>&Countable $c */
function needsIntCollectionParam($c): void {}

needsIntCollectionBound(new IntCollection());
needsIntCollectionParam(new IntCollection());

needsIntCollectionBound(new StringCollection());
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'StringCollection' does not satisfy bound 'Collection<int>&Countable'
needsIntCollectionParam(new StringCollection());
//                      ^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $c of needsIntCollectionParam() expects 'Collection<int>&Countable', got 'StringCollection'
===expect===
