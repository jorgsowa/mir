===description===
FP: `@template T of Collection<int>` rejected a valid concrete class that
fixedly implements `Collection<int>` — variance_compatible_across_hierarchy
bailed out on any concrete (non-generic) sub class before ever consulting
its `@implements Collection<int>` binding, because it treated an empty
`sub_params` (IntCollection isn't itself parameterized) as "nothing to
check" instead of "no OWN bindings, but an ancestor binding may still
apply".
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
interface Collection {}
/** @implements Collection<int> */
class IntCollection implements Collection {}

/**
 * @template T of Collection<int>
 * @param T $c
 */
function takesIntCollection($c): void {}

takesIntCollection(new IntCollection());
===expect===
