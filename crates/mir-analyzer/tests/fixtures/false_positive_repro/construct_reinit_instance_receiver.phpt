===description===
An explicit constructor re-init call ($obj->__construct()) resolves through a
receiver typed by an inherited property: the intentional DirectConstructorCall
lint fires (and nothing else — no UndefinedMethod/visibility false positives),
while the reference index still records the site under the owner (covered by
indexed_queries.rs; this pins the diagnostic surface).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Widget {
    public function __construct() {}
}

class Base {
    public function __construct(protected Widget $w) {}
}

class Caller extends Base {
    public function run(): void {
        $this->w->__construct();
    }
}
===expect===
DirectConstructorCall@13:8-13:31: Cannot call constructor of Widget directly
