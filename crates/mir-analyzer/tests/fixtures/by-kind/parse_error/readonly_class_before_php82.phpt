===description===
readonly classes require PHP 8.2.
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
readonly class A {
//<^^^^^^^^ ParseError: Parse error: 'readonly class' requires PHP 8.2 or higher [version-too-low]
    public function __construct(public int $x) {}
}
