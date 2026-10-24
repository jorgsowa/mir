===description===
Writes to static properties and static-call-stored taint resolve fully qualified and relative class names.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

class Reg {
    public static string $v = '';
    public static function put(string $s): void { self::$v = $s; }
}

function f(): void {
    \App\Reg::$v = $_GET['a'];
    echo \App\Reg::$v;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    namespace\Reg::$v = $_GET['b'];
    echo namespace\Reg::$v;
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function g(): void {
    \App\Reg::put($_GET['c']);
    echo Reg::$v;
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
