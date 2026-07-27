===description===
@taint-source on a static method call was only honored for a literal
class name (or self/static/parent) — `$class::getQuery()` through a
variable holding a known class-string fell through unhandled, unlike
the instance-method arm right above it, which already resolves a
variable receiver via `resolve_chained_receiver_type`.
===config===
suppress=MixedReturnStatement,MixedArrayAccess,MixedAssignment
===file===
<?php
class Request {
    /** @taint-source */
    public static function getQuery(): string {
        return $_GET['q'] ?? '';
    }
}

function test(string $requestCls): void {
    /** @var class-string<Request> $requestCls */
    echo $requestCls::getQuery();
}
===expect===
TaintedHtml@11:4-11:33: Tainted HTML output — possible XSS
