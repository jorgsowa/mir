===description===
@taint-source on a static method call was never honored -- is_expr_tainted
had a pairing for a plain instance method call but no StaticMethodCall arm
at all, unlike the read-side StaticPropertyAccess arm which already
handles self/static/parent.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Request {
    /** @taint-source */
    public static function getQuery(): string {
        return $_GET['q'] ?? '';
    }
}

echo Request::getQuery();
//<^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
