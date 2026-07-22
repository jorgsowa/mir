===description===
An untyped property (no native type hint, no @var docblock) that the
constructor unconditionally assigns a `new` expression to is no longer
`mixed` everywhere it's read — its type is inferred from the constructor's
own assignment, the same way method/function return types are inferred from
body analysis.
===config===
suppress=MissingPropertyType,UnusedParam
===file===
<?php
class ArrayCache {
    public function get(string $key): mixed { return null; }
}

class Repo {
    public $cache;

    public function __construct() {
        $this->cache = new ArrayCache();
    }

    public function read(): void {
        /** @mir-check $this->cache is ArrayCache */
        $_ = 1;
    }
}
===expect===
