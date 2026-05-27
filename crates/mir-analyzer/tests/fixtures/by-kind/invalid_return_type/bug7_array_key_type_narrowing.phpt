===description===
Array key type is preserved when building via assignment (Bug 7)
===file===
<?php

/**
 * @return array<class-string<\Throwable>, string>
 */
function indexErrors(): array
{
    /** @var list<class-string<\Throwable>> $classes */
    $classes = [\RuntimeException::class, \LogicException::class];

    $out = [];
    foreach ($classes as $cls) {
        $out[$cls] = $cls;
    }
    return $out;
}

/**
 * @return array<int, string>
 */
function simpleArray(): array
{
    $out = [];
    foreach ([1, 2, 3] as $key) {
        $out[$key] = "value";
    }
    return $out;
}

/**
 * @return array<string, int>
 */
function stringKeys(): array
{
    $out = [];
    foreach (["a", "b", "c"] as $key) {
        $out[$key] = 42;
    }
    return $out;
}
===expect===
