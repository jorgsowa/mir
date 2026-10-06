===description===
FP: a bounded template `@template T of Bound` with a `?T`/`T|null` parameter
must not reject a bare `null` argument — the `null` alternative in the union
already accounts for it, so this call gives no information about `T` at all
and must not be checked against `T`'s bound.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class NotBase {}

/**
 * @template T of Base
 * @param T|null $x
 * @return T|null
 */
function maybe_base($x) { return $x; }

maybe_base(null);
maybe_base(new Base());

/**
 * @template T of Base
 * @param ?T $x
 */
function maybe_base2($x) {}

maybe_base2(null);

// A real bound violation on the same union shape must still be caught.
maybe_base(new NotBase());
//<^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
