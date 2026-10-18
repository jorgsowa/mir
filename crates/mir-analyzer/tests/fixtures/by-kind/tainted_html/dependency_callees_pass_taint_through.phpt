===description===
A call into another Composer package is treated like a builtin: its result
carries its arguments' taint, but its body isn't summarized, so sinks and
sources inside it aren't followed.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:composer.json===
{
  "autoload": {
    "psr-4": {
      "Acme\\": "vendor/acme/text/src/"
    }
  }
}
===file:vendor/acme/text/src/Text.php===
<?php
namespace Acme;
class Text {
    public static function upper(string $s): string { return strtoupper($s); }
    public static function length(string $s): int { return strlen($s); }
    public static function show(string $s): void { echo $s; }
    public static function query(): string { return (string) $_GET['q']; }
}
===file:src/Page.php===
<?php
namespace App;
use Acme\Text;
class Page {
    public function render(string $name): void {
        echo Text::upper($name);
        echo Text::length($name);
        Text::show($name);
    }
}
function page(): void {
    echo Text::upper((string) $_GET['x']);
    echo Text::length((string) $_GET['x']);
    Text::show((string) $_GET['x']);
    echo Text::query();
}
===expect===
Page.php: TaintedHtml@12:4-12:42: Tainted HTML output — possible XSS
