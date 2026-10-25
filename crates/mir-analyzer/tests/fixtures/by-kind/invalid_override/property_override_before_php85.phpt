===description===
Before PHP 8.5 `#[Override]` may not target a property at all.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
final class C {
    #[Override]
//    ^^^^^^^^ CompileError: Attribute "Override" cannot target property (allowed targets: method) [attribute-target-mismatch]
    public int $extra = 0;
}
