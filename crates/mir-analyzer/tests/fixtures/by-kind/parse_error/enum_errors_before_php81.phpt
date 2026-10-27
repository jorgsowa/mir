===description===
Before PHP 8.1 an enum is only `version-too-low`; the parser's enum declaration errors are not reported next to it.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
enum Mixed: float {
//<^^^^ ParseError: 'enums' requires PHP 8.1 or higher [version-too-low]
    case A;
    case class = 1.0;
    public $x;
    abstract function f();
}
