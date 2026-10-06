===description===
Targets that depend on a runtime value are not resolved statically
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
function a_fn(string $name): int {
    include __DIR__ . '/' . $name . '.inc';
    return b_fn();
//  ^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//         ^^^^^^ UndefinedFunction: Function b_fn() is not defined
}
===file:b.inc===
<?php
function b_fn(): int { return 1; }
