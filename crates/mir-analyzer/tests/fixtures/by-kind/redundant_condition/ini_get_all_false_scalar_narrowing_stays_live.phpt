===description===
`ini_get_all(null, false)` values may be scalar, so `!is_scalar()` must not make following code unreachable.
===file===
<?php
foreach (ini_get_all(null, false) as $value) {
    if (!is_scalar($value)) {
        continue;
    }

    echo strtolower((string) $value);
}
===expect===
