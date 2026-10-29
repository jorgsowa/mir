===description===
Before PHP 8.6 writing a property of an enum case is only the temporary-expression compile error.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
enum Suit: string {
    case Hearts = 'h';

    final abstract public function label(): string;
//  ^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use the final modifier on an abstract method [abstract-final-conflict]
}
Suit::Hearts->value = 'x';
//<^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use temporary expression in write context [temporary-expression-write]
