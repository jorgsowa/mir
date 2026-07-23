===description===
@taint-sink on a static method's parameter was a complete no-op — only
instance ($obj->method()) calls checked taint_sink_params, never a static
method call reached via an explicit class name.
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
class Db {
    /** @taint-sink sql $sql */
    public static function run(string $sql): void {
    }
}

Db::run((string) $_GET["q"]);
===expect===
TaintedSql@8:0-8:28: Tainted SQL query — possible SQL injection
