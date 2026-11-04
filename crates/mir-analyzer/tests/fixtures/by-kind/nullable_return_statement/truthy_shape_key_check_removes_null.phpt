===description===
`if ($a['k'])` narrows the shape key's value to truthy
===file===
<?php
/** @param array{level: string|null} $config */
function f(array $config): string {
    if ($config['level']) {
        return $config['level'];
    }
    return 'x';
}
