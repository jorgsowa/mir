===description===
A callable array's receiver class isn't re-resolved through a `use` alias matching its first namespace segment.
===file===
<?php
namespace App { final class App {} }
namespace App\Platform { final class Runtime { public function invoke(): void {} } }
namespace App\Repository {
use App\App;
use App\Platform\Runtime;
function f(Runtime $r): void { call_user_func([$r, 'invoke']); }
}
