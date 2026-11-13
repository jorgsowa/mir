===description===
The key or value of a template-typed array passed back to the same template is unresolved until the template is bound, so it is not checked against the bound; a concrete type outside the bound still is.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template TValue of array<array|scalar>|scalar
 * @param TValue $value
 */
function resolve(mixed $value): mixed {
    if (is_array($value)) {
        $resolved = [];
        foreach ($value as $key => $item) {
            $resolvedKey = is_string($key) ? resolve($key) : $key;
            $resolved[] = [$resolvedKey, resolve($item)];
        }
        return $resolved;
    }
    return $value;
}

resolve(['a' => 1, 'b' => ['c' => true]]);
resolve('plain');
resolve(new \stdClass());
//<^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of resolve() expects 'array<int|string, array|scalar>|scalar', got 'stdClass'
