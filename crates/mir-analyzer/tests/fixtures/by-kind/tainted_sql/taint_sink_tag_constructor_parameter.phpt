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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
===expect===
