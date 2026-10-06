===description===
Nested shape returns still reject unrelated class-strings, missing or extra keys, and optional-vs-required mismatches
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class P {}
final class C extends P {}
final class Other {}

/** @return array<string, array{class-string<P>}> */
function unrelated(): array {
    return ['x' => [Other::class]];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'x': array{0: class-string<Other>}}' is not compatible with declared 'array<string, array{0: class-string<P>}>'
}

/** @return array<string, array{a: class-string<P>}> */
function missingKey(): array {
    return ['x' => []];
//  ^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'x': array{}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
}

/** @return array<string, array{a: class-string<P>}> */
function extraKey(): array {
    return ['x' => ['a' => C::class, 'b' => C::class]];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'x': array{'a': class-string<C>, 'b': class-string<C>}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
}

/** @return array<string, array{a: class-string<P>}> */
function wrongLeaf(): array {
    return ['x' => ['a' => 1]];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'x': array{'a': 1}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
}

/** @return array<string, array{a: class-string<C>}> */
function parentToChild(): array {
    return ['x' => ['a' => P::class]];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'x': array{'a': class-string<P>}}' is not compatible with declared 'array<string, array{'a': class-string<C>}>'
}
===expect===
