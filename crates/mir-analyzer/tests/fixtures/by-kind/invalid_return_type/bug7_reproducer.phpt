===description===
Bug 7 reproducer: array key type not widened to mixed
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
===expect===
