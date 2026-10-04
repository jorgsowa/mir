===description===
A template with a bound is judged by that bound inside a shape or list: a
bound that fits the expected value is accepted, one that cannot is rejected.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T of int */
class IntBox {
    /** @return T */
    public function get(): mixed { return 1; }
}

/** @param array{k?: int|string} $x */
function takeIntOrString(array $x): void {}

/** @param array{k?: string} $x */
function takeString(array $x): void {}

/** @param list<string> $x */
function takeStringList(array $x): void {}

takeIntOrString(['k' => (new IntBox)->get()]);
takeString(['k' => (new IntBox)->get()]);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $x of takeString() expects 'array{'k'?: string}', got 'array{'k': T}'
takeStringList([(new IntBox)->get()]);
//             ^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $x of takeStringList() expects 'list<string>', got 'array{0: T}'
===expect===
