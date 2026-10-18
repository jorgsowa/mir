===description===
A call through an interface or abstract method only takes flows from the
implementations it can summarize; one in another Composer package is described
by its annotations alone, like the abstract declaration, instead of passing
every argument through. Calling that implementation directly still treats it
like a builtin. The dependency's files are analyzed here, so its classes are
among the implementations.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:vendor/acme/kit/src/Store.php===
<?php
namespace Acme;
interface Store {
    public function get(string $key): string;
}
===file:vendor/acme/kit/src/ArrayStore.php===
<?php
namespace Acme;
class ArrayStore implements Store {
    /** @var array<string, string> */
    private array $items = [];
    public function get(string $key): string { return $this->items[$key] ?? ''; }
}
===file:vendor/acme/kit/src/Codec.php===
<?php
namespace Acme;
abstract class Codec {
    abstract public function decode(string $raw): string;
}
===file:vendor/acme/kit/src/Base64Codec.php===
<?php
namespace Acme;
class Base64Codec extends Codec {
    public function decode(string $raw): string { return (string) base64_decode($raw); }
}
===file:vendor/acme/kit/src/Escaper.php===
<?php
namespace Acme;
interface Escaper {
    public function escape(string $text): string;
}
===file:vendor/acme/kit/src/HtmlEscaper.php===
<?php
namespace Acme;
class HtmlEscaper implements Escaper {
    /** @psalm-taint-escape html */
    public function escape(string $text): string { return htmlspecialchars($text); }
}
===file:src/PlainCodec.php===
<?php
namespace App;
use Acme\Codec;
class PlainCodec extends Codec {
    public function decode(string $raw): string { return $raw; }
}
===file:src/Page.php===
<?php
namespace App;
use Acme\ArrayStore;
use Acme\Codec;
use Acme\Escaper;
use Acme\Store;
function page(Store $store, Codec $codec, Escaper $escaper): void {
    echo $store->get((string) $_GET['k']);
    $array = new ArrayStore();
    echo $array->get((string) $_GET['k']);
    echo $codec->decode((string) $_GET['c']);
    echo $escaper->escape((string) $_GET['e']);
    shell_exec($escaper->escape((string) $_GET['e']));
}
===expect===
Page.php: TaintedHtml@10:4-10:42: Tainted HTML output — possible XSS
Page.php: TaintedHtml@11:4-11:45: Tainted HTML output — possible XSS
Page.php: TaintedShell@13:4-13:53: Tainted shell command — possible command injection
