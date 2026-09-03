===description===
PHP permits static calls through an instance receiver ($obj::m()), including
receivers typed only by an inherited property. Must analyze clean — no
InvalidStaticInvocation/UndefinedMethod — and the reference index records the
site under the owner (covered by indexed_queries.rs; this pins diagnostics).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Widget {
    public static function make(): void {}
}

class Base {
    public function __construct(protected Widget $w) {}
}

class Caller extends Base {
    public function run(): void {
        $this->w::make();
    }
}
===expect===
