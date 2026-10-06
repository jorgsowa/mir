===description===
Same gap as the static-property hop, for a STATIC-METHOD-CALL hop instead:
`Factory::repo()->get('id')` — `resolve_chained_receiver_type` had no arm
for `StaticMethodCall`, so a taint-source method reached through an
intermediate static factory call fell through to `None` and the whole
expression was untainted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Param {
    /** @taint-source */
    public function get(string $key): string {
        return $_GET[$key] ?? '';
    }
}

class Factory {
    public static function repo(): Param {
        return new Param();
    }
}

function leak(): void {
    echo Factory::repo()->get('id');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
