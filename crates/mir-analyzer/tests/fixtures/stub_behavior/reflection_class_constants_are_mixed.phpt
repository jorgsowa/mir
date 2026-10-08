===description===
`ReflectionClass::getConstants()` / `getConstant()` return arbitrary constant values
(null, enum cases and objects included), so they are `mixed`, not `scalar|array<scalar>`;
a template bounded by `int|string` is not violated by them. A genuinely
wider-than-bound element type (`float|int`) is still reported.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Kind {
    public const A = 0;
    public const B = 1;
}

function nameOf(int $kind): string {
    $names = array_flip((new ReflectionClass(Kind::class))->getConstants());
    return $names[$kind];
}

function valueOf(): void {
    $reflection = new ReflectionClass(Kind::class);
    $all = $reflection->getConstants();
    /** @mir-check $all is array<string, mixed> */
    $one = $reflection->getConstant('A');
    /** @mir-check $one is mixed */
    echo count($all), gettype($one);
}

/** @param array<string, float|int> $values */
function flipFloats(array $values): void {
    array_flip($values);
//  ^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'TValue' inferred as 'float|int' does not satisfy bound 'int|string'
}