===description===
A bare `iterable` docblock type's decomposed array branch must key on the
true PHP array-key domain (int|string), not a literal `mixed` — regression
guard for the parser bug where `iterable`/`iterable<V>` built that key as
`TMixed`. Observable end-to-end: if the key were still `mixed`, passing the
foreach key to a strictly-typed `int|string` parameter would fire
MixedArgument; it must not.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function needsIntOrString(int|string $k): void {}

/** @param iterable $iter */
function walk($iter): void
{
    foreach ($iter as $k => $v) {
        needsIntOrString($k);
    }
}
===expect===
