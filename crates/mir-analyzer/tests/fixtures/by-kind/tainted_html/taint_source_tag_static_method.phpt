===description===
@taint-source on a static method call was never honored -- is_expr_tainted
had a pairing for a plain instance method call but no StaticMethodCall arm
at all, unlike the read-side StaticPropertyAccess arm which already
handles self/static/parent.
===config===
suppress=MixedReturnStatement,MixedArrayAccess
===file===
<?php
class Request {
    /** @taint-source */
    public static function getQuery(): string {
        return $_GET['q'] ?? '';
    }
}

echo Request::getQuery();
===expect===
TaintedHtml@9:0-9:25: Tainted HTML output — possible XSS
