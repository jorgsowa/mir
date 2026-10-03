===description===
@psalm-suppress / @mir-ignore InternalMethod on the statement covers instance and static calls; unsuppressed siblings still report.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Lib.php===
<?php
namespace Vendor\Lib;
class Cov {
    /** @internal */
    public function getReport(): int { return 1; }
    /** @internal */
    public static function make(): int { return 1; }
}
===file:Main.php===
<?php
namespace App;
use Vendor\Lib\Cov;
function instance(Cov $c): int {
    /** @psalm-suppress InternalMethod */
    $r = $c->getReport();
    return $r;
}
function comment_form(Cov $c): int {
    // @mir-ignore InternalMethod
    $r = $c->getReport();
    return $r;
}
function static_call(): int {
    /** @psalm-suppress InternalMethod */
    $r = Cov::make();
    return $r;
}
function returned(Cov $c): int {
    /** @psalm-suppress InternalMethod */
    return $c->getReport();
}
function unsuppressed(Cov $c): int {
    return $c->getReport();
//         ^^^^^^^^^^^^^^^ InternalMethod: Method Vendor\Lib\Cov::getReport() is marked @internal
}
===expect===
