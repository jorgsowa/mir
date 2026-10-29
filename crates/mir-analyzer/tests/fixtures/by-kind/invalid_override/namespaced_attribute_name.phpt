===description===
`#[Override]` names `\Override` only when it resolves there: in a namespace, an unqualified `Override` is that namespace's class.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
namespace App;

use Override as Checked;

#[\Attribute]
class Override {}

class Child {
    #[Override]
    public function local(): void {}

    #[Override]
    public int $prop = 0;

    #[\Override]
    public function qualified(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: App\Child::qualified() has #[\Override] attribute, but no matching parent method exists

    #[Checked]
    public function imported(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: App\Child::imported() has #[\Override] attribute, but no matching parent method exists
}
