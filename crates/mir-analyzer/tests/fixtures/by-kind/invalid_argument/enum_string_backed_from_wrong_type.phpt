===description===
String-backed enum ::from() rejects int argument
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum Color: string {
    case Red = 'r';
    case Green = 'g';
}

Color::from(42);
//          ^^ ArgumentTypeCoercion: Argument $value of from() expects 'string', got '42' — coercion may fail at runtime
