===description===
FP-I1: the `imagick` PECL extension (Imagick, ImagickDraw, ...) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

function load(string $path): Imagick {
    return new Imagick($path);
}

function filterConstant(): int {
    return Imagick::FILTER_LANCZOS;
}

function handle(ImagickException $e): string {
    return $e->getMessage();
}
===expect===
