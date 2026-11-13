===description===
`get_object_vars()` is keyed by property name, so its keys are strings.
===file===
<?php
/** @return list<string> */
function names(object $o): array {
    return array_keys(get_object_vars($o));
}
