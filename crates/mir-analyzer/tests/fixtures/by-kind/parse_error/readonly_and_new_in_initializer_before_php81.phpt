===description===
Readonly parameters and new in initializers require PHP 8.1.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class A {
    public function __construct(public readonly int $x, public ?\stdClass $o = new \stdClass()) {}
//                                     ^^^^^^^^ ParseError: 'readonly parameters' requires PHP 8.1 or higher [version-too-low]
//                                                                             ^^^^^^^^^^^^^^^ ParseError: 'new in initializers' requires PHP 8.1 or higher [version-too-low]
}
