===description===
A case body reached by falling through from the previous body keeps that body's assignments and the previous labels' subject values.
===file===
<?php
function kindOf(string $kind): int {
    $sep = null;
    switch ($kind) {
        case 's':
        case 'E':
            $sep = ';';
            // no break
        case 'O':
        case 'C':
            $sep ??= ':';
            if ('E' === $kind) {
                return 1;
            }
            if (';' === $sep) {
                return 2;
            }
            return 3;
    }
    return 0;
}
