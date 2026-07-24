===description===
`self::$items['id'] = $tainted;` — same array-element-write taint gap
as the instance-property sibling, for a static-property base.
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
class Cache {
    public static array $items = [];
    public static function remember(): void {
        self::$items['id'] = $_GET['id'];
        echo self::$items['id'];
    }
}
===expect===
TaintedHtml@6:8-6:32: Tainted HTML output — possible XSS
