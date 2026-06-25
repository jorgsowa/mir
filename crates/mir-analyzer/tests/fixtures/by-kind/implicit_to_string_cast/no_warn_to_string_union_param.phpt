===description===
No warning when __toString object is passed to a string|int union parameter
===config===
suppress=UnusedParam
===file===
<?php
class Tag {
    public function __toString(): string { return 'tag'; }
}

/**
 * @param string|int $value
 */
function format($value): void {}

format(new Tag());
===expect===
