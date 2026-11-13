===description===
A literal key written to an untyped array known to be non-empty (after `empty()`) is read
back with the written type.
===file===
<?php
/** @return string|bool */
function f(array $editor) {
    if (empty($editor)) {
        return '';
    }
    $editor['url'] = 'abc';
    $editor['url'] = str_replace('a', 'b', $editor['url']);
    return $editor['url'];
}
