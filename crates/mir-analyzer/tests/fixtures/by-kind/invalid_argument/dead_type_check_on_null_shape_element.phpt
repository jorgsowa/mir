===description===
`is_object()` on a shape element already narrowed to null makes its branch dead, as it does for a variable.
===file===
<?php
/** @param array{value?: string|list<string>} $values */
function f(array $values): string {
    if (!isset($values['value'])) { $values['value'] = null; }
    if (!is_array($values['value'])) {
        return is_object($values['value']) ? get_class($values['value']) : 'x';
    }
    return '';
}
