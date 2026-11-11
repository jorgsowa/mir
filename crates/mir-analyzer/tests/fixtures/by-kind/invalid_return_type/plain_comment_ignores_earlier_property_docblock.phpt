===description===
A plain comment before a statement doesn't pick up a docblock from further up
===file===
<?php
class Flag {
    /** @var bool */
    private $enabled = false;

    public function count(): int {
        /* plain comment */
        return 1;
    }

    public function name(): string {
        $name = 'x';
        /* plain comment */
        $name = strtoupper($name);
        /** @mir-check $name is non-empty-uppercase-string */
        return $name;
    }
}
