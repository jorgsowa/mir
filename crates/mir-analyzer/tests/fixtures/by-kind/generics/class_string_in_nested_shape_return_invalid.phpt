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
}

/** @return array<string, array{a: class-string<P>}> */
function missingKey(): array {
    return ['x' => []];
}

/** @return array<string, array{a: class-string<P>}> */
function extraKey(): array {
    return ['x' => ['a' => C::class, 'b' => C::class]];
}

/** @return array<string, array{a: class-string<P>}> */
function wrongLeaf(): array {
    return ['x' => ['a' => 1]];
}

/** @return array<string, array{a: class-string<C>}> */
function parentToChild(): array {
    return ['x' => ['a' => P::class]];
}
===expect===
InvalidReturnType@8:4-8:35: Return type 'array{'x': array{0: class-string<Other>}}' is not compatible with declared 'array<string, array{0: class-string<P>}>'
InvalidReturnType@13:4-13:23: Return type 'array{'x': array{}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
InvalidReturnType@18:4-18:55: Return type 'array{'x': array{'a': class-string<C>, 'b': class-string<C>}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
InvalidReturnType@23:4-23:31: Return type 'array{'x': array{'a': 1}}' is not compatible with declared 'array<string, array{'a': class-string<P>}>'
InvalidReturnType@28:4-28:38: Return type 'array{'x': array{'a': class-string<P>}}' is not compatible with declared 'array<string, array{'a': class-string<C>}>'
