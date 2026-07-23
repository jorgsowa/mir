===description===
@taint-sink on a constructor parameter was a complete no-op -- analyze_new
had no taint-sink check at all, unlike call/function.rs and call/method.rs
which both check it for their own call shapes.
===config===
suppress=MixedArrayAccess,UnusedParam,MissingConstructor
===file===
<?php
class Query {
    /** @taint-sink sql $sql */
    public function __construct(string $sql) {
    }
}

new Query((string) $_GET["q"]);
===expect===
TaintedSql@8:0-8:30: Tainted SQL query — possible SQL injection
