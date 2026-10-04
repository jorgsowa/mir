===description===
A partial user-stub interface keeps the real interface's own methods, constants and parent interfaces; the stub's members win on conflict.
===config===
<mir>
  <stubs>
    <file name="stubs/Handle.php"/>
  </stubs>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Base.php===
<?php
namespace App;

interface Base {
    public function inherited(): bool;
}
===file:src/Handle.php===
<?php
namespace App;

interface Handle extends Base {
    const LIMIT = 10;
    public function real(int $n): string;
    public function overridden(): int;
}
===file:src/Service.php===
<?php
namespace App;

class Service {
    public function run(): int { return 1; }
}
===file:stubs/Handle.php===
<?php
namespace App;

interface Handle {
    /** @return non-empty-string */
    public function stubbed(string $name);
    public function overridden(): string;
}
===file:main.php===
<?php
use App\Base;
use App\Handle;
use App\Service;

function go(Handle $h): void {
    $a = $h->real(1);
    /** @mir-check $a is string */
    $b = $h->stubbed('x');
    /** @mir-check $b is non-empty-string */
    $c = $h->overridden();
    /** @mir-check $c is string */
    $d = $h->inherited();
    /** @mir-check $d is bool */
    $e = Handle::LIMIT;
    /** @mir-check $e is 10 */
    $h->missing();
}

/** @param Handle&Service $m */
function intersection(object $m): void {
    $m->real(1);
    $m->run();
    $m->missing();
}

function asParent(Handle $h): Base {
    return $h;
}
===expect===
main.php: UndefinedMethod@24:4-24:17: Method App\Handle&App\Service::missing() does not exist
