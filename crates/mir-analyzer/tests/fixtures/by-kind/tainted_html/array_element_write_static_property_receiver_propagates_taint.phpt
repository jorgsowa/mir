===description===
`self::$items['id'] = $tainted;` — same array-element-write taint gap
as the instance-property sibling, for a static-property base.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Cache {
    public static array $items = [];
    public static function remember(): void {
        self::$items['id'] = $_GET['id'];
        echo self::$items['id'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
