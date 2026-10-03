===description===
Includes nested in conditionals and function bodies are still discovered
===config===
<mir>
  <projectFiles>
    <file name="a.module"/>
  </projectFiles>
  <fileExtensions>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:a.module===
<?php
function a_fn(bool $f): int {
    if ($f) {
        require_once __DIR__ . '/b.inc';
    }
    return b_fn();
}
===file:b.inc===
<?php
function b_fn(): int { return 1; }
===expect===
