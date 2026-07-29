===description===
FP-C6: the `ast` PECL extension (nikic/php-ast) has no vendored stub, so
`ast\parse_code`/`ast\parse_file`/`ast\Node` were all reported undefined even
though `PhpStormStubsMap.php` already lists them (pointing at a stub file that
didn't exist on disk).
===config===
suppress=UnusedParam
===file===
<?php

function inspect(string $code): void {
    $node = ast\parse_code($code, 90);
    echo $node->kind;
    echo $node->flags;
    echo $node->lineno;
    echo ast\get_kind_name($node->kind);
}

function inspectFile(string $path): ast\Node {
    return ast\parse_file($path, 90);
}
===expect===
