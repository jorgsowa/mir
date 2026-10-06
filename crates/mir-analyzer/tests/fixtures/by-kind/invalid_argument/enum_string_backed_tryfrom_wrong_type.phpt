===description===
String-backed enum ::tryFrom() rejects int argument
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum Status: string {
    case Active = 'active';
}

Status::tryFrom(123);
//              ^^^ ArgumentTypeCoercion: Argument $value of tryFrom() expects 'string', got '123' — coercion may fail at runtime
