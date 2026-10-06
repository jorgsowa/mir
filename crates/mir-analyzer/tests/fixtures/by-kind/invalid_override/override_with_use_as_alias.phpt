===description===
FP-I: `use Foo as Bar` alias not resolved for override checks. A class that
extends an aliased parent and marks a method with #[\Override] must not emit
InvalidOverride when the method exists on the aliased parent.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file:base.php===
<?php

namespace App\Foundation;

class Renderer {
    public function render(): string { return ''; }
}
===file:child.php===
<?php

namespace App;

use App\Foundation\Renderer as BaseRenderer;

class HtmlRenderer extends BaseRenderer {
    #[\Override]
    public function render(): string { return '<html>'; }
}
